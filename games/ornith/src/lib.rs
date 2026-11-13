//! Surviving Ornith: a compact single-player survival vertical slice powered by UBE.

use std::f32::consts::PI;
use std::time::Duration;

use engine::{
    BoxCollider, CameraMode, Entity, GameAction, GameFeedback, GameHud, GamePlugin, HudSlot,
    ModelReference, PlayerSnapshot, Scene, Transform,
};

pub const SOUND_EFFECTS: &[&str] = &[
    "rifle", "pistol", "reload", "mining", "chopping", "building", "taming", "eating", "hurt",
    "empty",
];
const HIDDEN_Y: f32 = -100.0;
const RESPAWN: [f32; 3] = [0.0, 0.91, 6.0];
const BUILD_GRID: f32 = 3.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Tool {
    Rifle,
    Pistol,
    Pickaxe,
    Axe,
}

impl Tool {
    fn slot(self) -> u8 {
        match self {
            Self::Rifle => 1,
            Self::Pistol => 2,
            Self::Pickaxe => 3,
            Self::Axe => 4,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Rifle => "Rifle",
            Self::Pistol => "Pistol",
            Self::Pickaxe => "Pickaxe",
            Self::Axe => "Axe",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct WeaponSpec {
    magazine: u8,
    reload_seconds: f32,
    cooldown_seconds: f32,
    range: f32,
    damage: f32,
    spread: f32,
    sound: &'static str,
}

fn weapon_spec(tool: Tool) -> Option<WeaponSpec> {
    match tool {
        Tool::Rifle => Some(WeaponSpec {
            magazine: 1,
            reload_seconds: 2.4,
            cooldown_seconds: 0.8,
            range: 80.0,
            damage: 100.0,
            spread: 0.001,
            sound: "rifle",
        }),
        Tool::Pistol => Some(WeaponSpec {
            magazine: 6,
            reload_seconds: 1.05,
            cooldown_seconds: 0.22,
            range: 22.0,
            damage: 34.0,
            spread: 0.055,
            sound: "pistol",
        }),
        Tool::Pickaxe | Tool::Axe => None,
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Inventory {
    wood: u32,
    stone: u32,
    meat: u32,
    berries: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NodeKind {
    Tree,
    Ore,
    Berry,
}

#[derive(Clone, Debug)]
struct ResourceNode {
    entity: Entity,
    kind: NodeKind,
    position: [f32; 3],
    hits: u8,
    alive: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AnimalKind {
    Wolf,
    Cow,
    Hawk,
}

impl AnimalKind {
    fn name(self) -> &'static str {
        match self {
            Self::Wolf => "Wolf",
            Self::Cow => "Cow",
            Self::Hawk => "Hawk",
        }
    }

    fn food(self) -> &'static str {
        match self {
            Self::Cow => "berries",
            Self::Wolf | Self::Hawk => "meat",
        }
    }

    fn model(self) -> &'static str {
        match self {
            Self::Wolf => "wolf",
            Self::Cow => "cow",
            Self::Hawk => "hawk",
        }
    }

    fn target_height(self) -> f32 {
        match self {
            Self::Wolf => 0.9,
            Self::Cow => 1.1,
            Self::Hawk => 0.4,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Sex {
    Female,
    Male,
}

impl Sex {
    fn name(self) -> &'static str {
        match self {
            Self::Female => "female",
            Self::Male => "male",
        }
    }
}

#[derive(Clone, Debug)]
struct Animal {
    entity: Entity,
    kind: AnimalKind,
    sex: Sex,
    position: [f32; 3],
    home: [f32; 3],
    health: f32,
    alive: bool,
    adult: bool,
    age_seconds: f32,
    tamed: bool,
    follow: bool,
    attack_cooldown: f32,
    breed_cooldown: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BuildKind {
    Wall,
    Foundation,
}

impl BuildKind {
    fn name(self) -> &'static str {
        match self {
            Self::Wall => "Wall",
            Self::Foundation => "Foundation",
        }
    }

    fn cost(self) -> (u32, u32) {
        match self {
            Self::Wall => (4, 2),
            Self::Foundation => (6, 1),
        }
    }

    fn model(self) -> &'static str {
        match self {
            Self::Wall => "wall",
            Self::Foundation => "foundation",
        }
    }
}

#[derive(Clone, Debug)]
struct BuildPiece {
    entity: Entity,
    kind: BuildKind,
    position: [f32; 3],
    active: bool,
}

/// Application-owned game rules. UBE owns native input, physics, rendering, and audio output.
pub struct OrnithGame {
    resources: Vec<ResourceNode>,
    animals: Vec<Animal>,
    builds: Vec<BuildPiece>,
    viewmodels: [Entity; 4],
    avatar: Entity,
    inventory: Inventory,
    selected: Tool,
    rifle_loaded: u8,
    pistol_loaded: u8,
    reloading: Option<(Tool, f32)>,
    action_cooldown: f32,
    shot_index: u32,
    build_mode: bool,
    build_kind: BuildKind,
    companions_follow: bool,
    health: f32,
    hunger: f32,
    elapsed: f32,
    intro_seconds: f32,
    status: String,
    last_player: PlayerSnapshot,
    built_count: u32,
}

/// Creates the complete bounded island and its preallocated interactive-object pools.
pub fn new_world() -> (Scene, OrnithGame) {
    let mut scene = Scene::new();
    spawn_model(&mut scene, "terrain", Transform::default());
    spawn_model(
        &mut scene,
        "water",
        Transform::default().with_translation([0.0, -0.85, 0.0]),
    );
    let ground = scene.spawn(Transform::default().with_translation([0.0, -1.0, 0.0]));
    set_collider(&mut scene, ground, Some([21.0, 1.0, 21.0]));

    let tree_positions = [
        [-4.0, 0.0, 0.0],
        [5.0, 0.0, -3.0],
        [-8.0, 0.0, -8.0],
        [10.0, 0.0, 3.0],
        [-12.0, 0.0, 5.0],
        [2.0, 0.0, -13.0],
        [13.0, 0.0, -11.0],
        [-15.0, 0.0, -12.0],
        [8.0, 0.0, 12.0],
        [-8.0, 0.0, 13.0],
        [15.0, 0.0, 10.0],
        [-15.0, 0.0, 11.0],
    ];
    let ore_positions = [
        [3.0, 0.0, 1.0],
        [-6.0, 0.0, -3.0],
        [8.0, 0.0, -8.0],
        [-11.0, 0.0, -12.0],
        [13.0, 0.0, 5.0],
        [-13.0, 0.0, 9.0],
        [4.0, 0.0, 13.0],
        [-3.0, 0.0, -15.0],
    ];
    let berry_positions = [
        [-2.5, 0.0, 2.0],
        [6.0, 0.0, 4.0],
        [-7.0, 0.0, 7.0],
        [10.0, 0.0, -2.0],
        [-11.0, 0.0, -5.0],
        [3.0, 0.0, -9.0],
    ];
    let mut resources = Vec::new();
    for (index, position) in tree_positions.into_iter().enumerate() {
        resources.push(spawn_resource(&mut scene, NodeKind::Tree, position, index));
    }
    for (index, position) in ore_positions.into_iter().enumerate() {
        resources.push(spawn_resource(&mut scene, NodeKind::Ore, position, index));
    }
    for (index, position) in berry_positions.into_iter().enumerate() {
        resources.push(spawn_resource(&mut scene, NodeKind::Berry, position, index));
    }

    // Seeded-looking decorative placement is fixed so screenshots and tests are reproducible.
    for index in 0..32 {
        let angle = index as f32 * 2.399_963;
        let radius = 4.0 + (index % 7) as f32 * 1.8;
        let position = [angle.cos() * radius, 0.0, angle.sin() * radius];
        spawn_model(
            &mut scene,
            "flower",
            Transform::default()
                .with_translation(position)
                .with_rotation_y(angle)
                .with_scale(0.8 + (index % 3) as f32 * 0.15),
        );
    }
    for index in 0..7 {
        spawn_model(
            &mut scene,
            "cloud",
            Transform::default()
                .with_translation([
                    -28.0 + index as f32 * 10.0,
                    12.0 + (index % 3) as f32 * 2.0,
                    -28.0 + (index % 4) as f32 * 16.0,
                ])
                .with_scale(1.2 + (index % 2) as f32 * 0.5),
        );
    }

    let animal_specs = [
        (AnimalKind::Wolf, Sex::Female, [8.0, 0.0, -9.0]),
        (AnimalKind::Wolf, Sex::Male, [13.0, 0.0, -14.0]),
        (AnimalKind::Cow, Sex::Female, [-6.0, 0.0, -6.0]),
        (AnimalKind::Cow, Sex::Male, [-11.0, 0.0, 1.0]),
        (AnimalKind::Hawk, Sex::Female, [-2.0, 2.8, -10.0]),
        (AnimalKind::Hawk, Sex::Male, [5.0, 3.2, -15.0]),
    ];
    let mut animals = Vec::new();
    for (kind, sex, position) in animal_specs {
        animals.push(spawn_animal(&mut scene, kind, sex, position, true, true));
    }
    // Unborn offspring share the same fixed-capacity renderer pool as adults.
    for (kind, _, _) in animal_specs {
        animals.push(spawn_animal(
            &mut scene,
            kind,
            if animals.len() % 2 == 0 {
                Sex::Female
            } else {
                Sex::Male
            },
            [0.0, HIDDEN_Y, 0.0],
            false,
            false,
        ));
    }

    let mut builds = Vec::new();
    for index in 0..16 {
        let kind = if index % 2 == 0 {
            BuildKind::Wall
        } else {
            BuildKind::Foundation
        };
        let entity = spawn_model(
            &mut scene,
            kind.model(),
            Transform::default().with_translation([0.0, HIDDEN_Y, 0.0]),
        );
        builds.push(BuildPiece {
            entity,
            kind,
            position: [0.0; 3],
            active: false,
        });
    }

    let viewmodels = [
        spawn_model(&mut scene, "rifle", hidden_transform()),
        spawn_model(&mut scene, "pistol", hidden_transform()),
        spawn_model(&mut scene, "pickaxe", hidden_transform()),
        spawn_model(&mut scene, "axe", hidden_transform()),
    ];
    let avatar = spawn_model(&mut scene, "survivor", hidden_transform());
    let game = OrnithGame {
        resources,
        animals,
        builds,
        viewmodels,
        avatar,
        inventory: Inventory {
            wood: 0,
            stone: 0,
            meat: 4,
            berries: 4,
        },
        selected: Tool::Rifle,
        rifle_loaded: 1,
        pistol_loaded: 6,
        reloading: None,
        action_cooldown: 0.0,
        shot_index: 0,
        build_mode: false,
        build_kind: BuildKind::Wall,
        companions_follow: true,
        health: 100.0,
        hunger: 100.0,
        elapsed: 0.0,
        intro_seconds: 8.0,
        status: "Landfall on Ornith Island. Your story starts here.".into(),
        last_player: PlayerSnapshot {
            position: RESPAWN,
            yaw: 0.0,
            pitch: 0.0,
            camera_mode: CameraMode::FirstPerson,
        },
        built_count: 0,
    };
    (scene, game)
}

fn hidden_transform() -> Transform {
    Transform::default().with_translation([0.0, HIDDEN_Y, 0.0])
}

fn model_path(name: &str) -> String {
    format!("assets/ornith/models/{name}.glb")
}

fn spawn_model(scene: &mut Scene, name: &str, transform: Transform) -> Entity {
    let entity = scene.spawn(transform);
    let reference = ModelReference::new(model_path(name)).expect("Ornith asset paths are valid");
    assert!(scene.set_model(entity, Some(reference)));
    entity
}

fn set_collider(scene: &mut Scene, entity: Entity, extents: Option<[f32; 3]>) {
    let collider = extents.map(|value| BoxCollider::new(value).expect("valid Ornith collider"));
    assert!(scene.set_collider(entity, collider));
}

fn spawn_resource(
    scene: &mut Scene,
    kind: NodeKind,
    position: [f32; 3],
    index: usize,
) -> ResourceNode {
    let model = match kind {
        NodeKind::Tree => "tree",
        NodeKind::Ore => "ore",
        NodeKind::Berry => "berry",
    };
    let entity = spawn_model(
        scene,
        model,
        Transform::default()
            .with_translation(position)
            .with_rotation_y(index as f32 * 1.7),
    );
    match kind {
        NodeKind::Tree => set_collider(scene, entity, Some([0.4, 1.5, 0.4])),
        NodeKind::Ore => set_collider(scene, entity, Some([0.75, 0.6, 0.68])),
        NodeKind::Berry => set_collider(scene, entity, None),
    }
    ResourceNode {
        entity,
        kind,
        position,
        hits: 3,
        alive: true,
    }
}

fn spawn_animal(
    scene: &mut Scene,
    kind: AnimalKind,
    sex: Sex,
    position: [f32; 3],
    alive: bool,
    adult: bool,
) -> Animal {
    let entity = spawn_model(
        scene,
        kind.model(),
        Transform::default()
            .with_translation(position)
            .with_scale(if adult { 1.0 } else { 0.55 }),
    );
    Animal {
        entity,
        kind,
        sex,
        position,
        home: position,
        health: if kind == AnimalKind::Cow {
            120.0
        } else {
            100.0
        },
        alive,
        adult,
        age_seconds: 0.0,
        tamed: false,
        follow: false,
        attack_cooldown: 0.0,
        breed_cooldown: 0.0,
    }
}

impl OrnithGame {
    fn sound(name: &str) -> GameFeedback {
        GameFeedback {
            sound_effects: vec![name.into()],
            ..GameFeedback::default()
        }
    }

    fn loaded_mut(&mut self, tool: Tool) -> Option<&mut u8> {
        match tool {
            Tool::Rifle => Some(&mut self.rifle_loaded),
            Tool::Pistol => Some(&mut self.pistol_loaded),
            Tool::Pickaxe | Tool::Axe => None,
        }
    }

    fn loaded(&self, tool: Tool) -> u8 {
        match tool {
            Tool::Rifle => self.rifle_loaded,
            Tool::Pistol => self.pistol_loaded,
            Tool::Pickaxe | Tool::Axe => 0,
        }
    }

    fn fire(&mut self, scene: &mut Scene, player: PlayerSnapshot) -> GameFeedback {
        let Some(spec) = weapon_spec(self.selected) else {
            return GameFeedback::default();
        };
        if self.action_cooldown > 0.0 {
            return GameFeedback::default();
        }
        if self.reloading.is_some() {
            self.status = "Reloading — stay ready.".into();
            return GameFeedback::default();
        }
        if self.loaded(self.selected) == 0 {
            self.status = format!("{} empty. Press R to reload.", self.selected.name());
            return Self::sound("empty");
        }
        *self.loaded_mut(self.selected).unwrap() -= 1;
        self.action_cooldown = spec.cooldown_seconds;
        let pattern = [0.0, 0.75, -0.6, 1.0, -0.9, 0.35, -0.2, 0.55];
        let offset = pattern[self.shot_index as usize % pattern.len()] * spec.spread;
        self.shot_index = self.shot_index.wrapping_add(1);
        let spread_player = PlayerSnapshot {
            yaw: player.yaw + offset,
            pitch: player.pitch
                + pattern[(self.shot_index as usize + 2) % pattern.len()] * spec.spread * 0.45,
            ..player
        };
        if let Some(index) = self.aimed_animal(spread_player, spec.range, 0.91) {
            self.damage_animal(scene, index, spec.damage);
        } else {
            self.status = format!("{} fired. No clean hit.", self.selected.name());
        }
        Self::sound(spec.sound)
    }

    fn reload(&mut self) -> GameFeedback {
        let Some(spec) = weapon_spec(self.selected) else {
            self.status = "Select the rifle or pistol to reload.".into();
            return GameFeedback::default();
        };
        if self.loaded(self.selected) == spec.magazine {
            self.status = format!("{} is already loaded.", self.selected.name());
            return GameFeedback::default();
        }
        self.reloading = Some((self.selected, spec.reload_seconds));
        self.status = format!(
            "Reloading {} — {:.1}s",
            self.selected.name(),
            spec.reload_seconds
        );
        Self::sound("reload")
    }

    fn harvest(&mut self, scene: &mut Scene, player: PlayerSnapshot) -> GameFeedback {
        if self.action_cooldown > 0.0 {
            return GameFeedback::default();
        }
        let expected = match self.selected {
            Tool::Pickaxe => NodeKind::Ore,
            Tool::Axe => NodeKind::Tree,
            Tool::Rifle | Tool::Pistol => return GameFeedback::default(),
        };
        let Some(index) = self.aimed_resource(player, 4.2, Some(expected)) else {
            self.status = format!(
                "Aim at {} within reach.",
                if expected == NodeKind::Ore {
                    "an ore rock"
                } else {
                    "a tree"
                }
            );
            return GameFeedback::default();
        };
        self.action_cooldown = 0.34;
        let node = &mut self.resources[index];
        node.hits = node.hits.saturating_sub(1);
        let sound = if expected == NodeKind::Ore {
            "mining"
        } else {
            "chopping"
        };
        let mut feedback = Self::sound(sound);
        if node.hits == 0 {
            node.alive = false;
            scene.set_local_transform(node.entity, hidden_transform());
            scene.set_collider(node.entity, None);
            if expected == NodeKind::Ore {
                self.inventory.stone += 4;
                self.status = "Ore recovered: +4 stone. The rock is depleted.".into();
            } else {
                self.inventory.wood += 6;
                self.status = "Timber! +6 wood. The tree is cleared.".into();
            }
            feedback.physics_changed = true;
        } else {
            self.status = format!("{} more clean hit(s).", node.hits);
        }
        feedback
    }

    fn interact(&mut self, scene: &mut Scene, player: PlayerSnapshot) -> GameFeedback {
        if let Some(index) = self.aimed_animal(player, 4.5, 0.3) {
            let kind = self.animals[index].kind;
            if !self.animals[index].tamed {
                if !self.consume_food(kind, 1) {
                    self.status = format!("Need 1 {} to tame this {}.", kind.food(), kind.name());
                    return GameFeedback::default();
                }
                self.animals[index].tamed = true;
                self.animals[index].follow = self.companions_follow;
                self.status = format!("{} tamed! It will follow and assist you.", kind.name());
                return Self::sound("taming");
            }
            if !self.animals[index].adult {
                self.status = "This youngster is still growing.".into();
                return GameFeedback::default();
            }
            if self.animals[index].breed_cooldown > 0.0 {
                self.status = format!(
                    "Breeding recovers in {:.0}s.",
                    self.animals[index].breed_cooldown
                );
                return GameFeedback::default();
            }
            let partner = self
                .animals
                .iter()
                .enumerate()
                .find(|(other_index, animal)| {
                    *other_index != index
                        && animal.alive
                        && animal.tamed
                        && animal.adult
                        && animal.kind == kind
                        && animal.sex != self.animals[index].sex
                        && animal.breed_cooldown <= 0.0
                        && distance(animal.position, self.animals[index].position) <= 5.0
                })
                .map(|(other_index, _)| other_index);
            let Some(partner) = partner else {
                self.status = format!(
                    "Bring a tamed adult {} of the opposite sex nearby.",
                    kind.name()
                );
                return GameFeedback::default();
            };
            let Some(baby) = self
                .animals
                .iter()
                .position(|animal| !animal.alive && !animal.adult && animal.kind == kind)
            else {
                self.status = "The island nursery is full for this test slice.".into();
                return GameFeedback::default();
            };
            if !self.consume_food(kind, 2) {
                self.status = format!("Breeding needs 2 {}.", kind.food());
                return GameFeedback::default();
            }
            let midpoint = interpolate(
                self.animals[index].position,
                self.animals[partner].position,
                0.5,
            );
            let position = [midpoint[0] + 0.8, midpoint[1], midpoint[2] + 0.5];
            self.animals[index].breed_cooldown = 45.0;
            self.animals[partner].breed_cooldown = 45.0;
            self.animals[baby].alive = true;
            self.animals[baby].tamed = true;
            self.animals[baby].follow = true;
            self.animals[baby].health = 100.0;
            self.animals[baby].age_seconds = 0.0;
            self.animals[baby].position = position;
            self.animals[baby].home = position;
            scene.set_local_transform(
                self.animals[baby].entity,
                Transform::default()
                    .with_translation(position)
                    .with_scale(0.55),
            );
            self.status = format!("A baby {} was born! It matures in 45 seconds.", kind.name());
            return Self::sound("taming");
        }
        if let Some(index) = self.aimed_resource(player, 3.8, Some(NodeKind::Berry)) {
            let node = &mut self.resources[index];
            node.alive = false;
            scene.set_local_transform(node.entity, hidden_transform());
            self.inventory.berries += 4;
            self.status = "Foraged +4 berries. Cows love these.".into();
            return Self::sound("eating");
        }
        self.status = "Nothing close enough to interact with.".into();
        GameFeedback::default()
    }

    fn consume_food(&mut self, kind: AnimalKind, amount: u32) -> bool {
        let available = match kind {
            AnimalKind::Cow => &mut self.inventory.berries,
            AnimalKind::Wolf | AnimalKind::Hawk => &mut self.inventory.meat,
        };
        if *available < amount {
            false
        } else {
            *available -= amount;
            true
        }
    }

    fn place_build(&mut self, scene: &mut Scene, player: PlayerSnapshot) -> GameFeedback {
        let (wood, stone) = self.build_kind.cost();
        if self.inventory.wood < wood || self.inventory.stone < stone {
            self.status = format!(
                "{} needs {wood} wood + {stone} stone.",
                self.build_kind.name()
            );
            return GameFeedback::default();
        }
        let forward = [player.yaw.sin(), 0.0, -player.yaw.cos()];
        let position = [
            ((player.position[0] + forward[0] * 3.2) / BUILD_GRID).round() * BUILD_GRID,
            0.0,
            ((player.position[2] + forward[2] * 3.2) / BUILD_GRID).round() * BUILD_GRID,
        ];
        if position.iter().any(|value| value.abs() > 18.0) {
            self.status = "Build on the island, away from the deep water.".into();
            return GameFeedback::default();
        }
        if horizontal_distance(position, player.position) < 2.0 {
            self.status = "Step back — that placement would overlap you.".into();
            return GameFeedback::default();
        }
        if self
            .builds
            .iter()
            .any(|piece| piece.active && horizontal_distance(piece.position, position) < 1.5)
        {
            self.status = "Another structure already occupies that grid cell.".into();
            return GameFeedback::default();
        }
        let Some(index) = self
            .builds
            .iter()
            .position(|piece| !piece.active && piece.kind == self.build_kind)
        else {
            self.status = format!(
                "No more {} pieces in this test pool.",
                self.build_kind.name()
            );
            return GameFeedback::default();
        };
        self.inventory.wood -= wood;
        self.inventory.stone -= stone;
        let piece = &mut self.builds[index];
        piece.active = true;
        piece.position = position;
        scene.set_local_transform(
            piece.entity,
            Transform::default().with_translation(position),
        );
        match piece.kind {
            BuildKind::Wall => set_collider(scene, piece.entity, Some([1.45, 1.2, 0.15])),
            BuildKind::Foundation => set_collider(scene, piece.entity, Some([1.45, 0.15, 1.45])),
        }
        self.built_count += 1;
        self.status = format!(
            "{} placed. -{wood} wood, -{stone} stone.",
            piece.kind.name()
        );
        GameFeedback {
            physics_changed: true,
            sound_effects: vec!["building".into()],
            respawn: None,
        }
    }

    fn eat(&mut self) -> GameFeedback {
        if self.inventory.meat > 0 {
            self.inventory.meat -= 1;
            self.hunger = (self.hunger + 35.0).min(100.0);
            self.health = (self.health + 5.0).min(100.0);
            self.status = "Cooked meat eaten: +35 hunger, +5 health.".into();
        } else if self.inventory.berries > 0 {
            self.inventory.berries -= 1;
            self.hunger = (self.hunger + 18.0).min(100.0);
            self.status = "Berries eaten: +18 hunger.".into();
        } else {
            self.status = "No food. Forage berries or hunt an animal.".into();
            return GameFeedback::default();
        }
        Self::sound("eating")
    }

    fn aimed_resource(
        &self,
        player: PlayerSnapshot,
        range: f32,
        kind: Option<NodeKind>,
    ) -> Option<usize> {
        self.resources
            .iter()
            .enumerate()
            .filter(|(_, node)| node.alive && kind.is_none_or(|expected| expected == node.kind))
            .filter_map(|(index, node)| {
                let height = match node.kind {
                    NodeKind::Tree => 1.8,
                    NodeKind::Ore => 0.6,
                    NodeKind::Berry => 0.6,
                };
                aim_score(player, add(node.position, [0.0, height, 0.0]), range, 0.25)
                    .map(|score| (index, score))
            })
            .min_by(|left, right| left.1.total_cmp(&right.1))
            .map(|(index, _)| index)
    }

    fn aimed_animal(&self, player: PlayerSnapshot, range: f32, minimum_dot: f32) -> Option<usize> {
        self.animals
            .iter()
            .enumerate()
            .filter(|(_, animal)| animal.alive)
            .filter_map(|(index, animal)| {
                aim_score(
                    player,
                    add(animal.position, [0.0, animal.kind.target_height(), 0.0]),
                    range,
                    minimum_dot,
                )
                .map(|score| (index, score))
            })
            .min_by(|left, right| left.1.total_cmp(&right.1))
            .map(|(index, _)| index)
    }

    fn damage_animal(&mut self, scene: &mut Scene, index: usize, damage: f32) {
        if !self.animals[index].alive {
            return;
        }
        self.animals[index].health -= damage;
        if self.animals[index].health <= 0.0 {
            let kind = self.animals[index].kind;
            self.animals[index].alive = false;
            scene.set_local_transform(self.animals[index].entity, hidden_transform());
            let meat = if kind == AnimalKind::Cow { 3 } else { 2 };
            self.inventory.meat += meat;
            self.status = format!(
                "{} hunted. +{meat} meat for food, taming, or breeding.",
                kind.name()
            );
        } else {
            self.status = format!(
                "{} hit — {:.0} health remains.",
                self.animals[index].kind.name(),
                self.animals[index].health
            );
        }
    }

    fn update_animals(&mut self, scene: &mut Scene, dt: f32, player: PlayerSnapshot) -> bool {
        let mut attacks = Vec::new();
        let mut player_damage = 0.0;
        for index in 0..self.animals.len() {
            if !self.animals[index].alive {
                continue;
            }
            self.animals[index].attack_cooldown =
                (self.animals[index].attack_cooldown - dt).max(0.0);
            self.animals[index].breed_cooldown = (self.animals[index].breed_cooldown - dt).max(0.0);
            if !self.animals[index].adult {
                self.animals[index].age_seconds += dt;
                if self.animals[index].age_seconds >= 45.0 {
                    self.animals[index].adult = true;
                }
            }
            let kind = self.animals[index].kind;
            let tamed = self.animals[index].tamed;
            let follow = self.animals[index].follow;
            let position = self.animals[index].position;
            let companion_target = if tamed && matches!(kind, AnimalKind::Wolf | AnimalKind::Hawk) {
                self.animals
                    .iter()
                    .enumerate()
                    .filter(|(other, animal)| {
                        *other != index
                            && animal.alive
                            && !animal.tamed
                            && animal.kind == AnimalKind::Wolf
                    })
                    .map(|(other, animal)| (other, distance(position, animal.position)))
                    .filter(|(_, range)| *range < 11.0)
                    .min_by(|left, right| left.1.total_cmp(&right.1))
                    .map(|(other, _)| other)
            } else {
                None
            };
            let mut destination = None;
            if let Some(target) = companion_target {
                let target_position = self.animals[target].position;
                let range = distance(position, target_position);
                if range > 1.5 {
                    destination = Some(target_position);
                } else if self.animals[index].attack_cooldown <= 0.0 {
                    attacks.push((target, if kind == AnimalKind::Wolf { 24.0 } else { 16.0 }));
                    self.animals[index].attack_cooldown = 0.9;
                }
            } else if tamed && follow {
                let offset = if index % 2 == 0 { 1.8 } else { -1.8 };
                let target = [
                    player.position[0] + offset,
                    if kind == AnimalKind::Hawk {
                        player.position[1] + 1.8
                    } else {
                        0.0
                    },
                    player.position[2] + 2.0,
                ];
                if distance(position, target) > 2.3 {
                    destination = Some(target);
                }
            } else if !tamed
                && kind == AnimalKind::Wolf
                && distance(position, player.position) < 9.0
            {
                let range = distance(position, player.position);
                if range > 1.45 {
                    destination = Some(player.position);
                } else if self.animals[index].attack_cooldown <= 0.0 {
                    player_damage += 8.0;
                    self.animals[index].attack_cooldown = 1.25;
                }
            } else if !tamed {
                let phase = self.elapsed * 0.22 + index as f32 * 1.9;
                let home = self.animals[index].home;
                let target = [
                    home[0] + phase.sin() * 2.0,
                    home[1],
                    home[2] + phase.cos() * 2.0,
                ];
                if distance(position, target) > 0.4 {
                    destination = Some(target);
                }
            }
            if let Some(target) = destination {
                let speed = if tamed {
                    2.8
                } else if kind == AnimalKind::Wolf {
                    1.3
                } else {
                    0.45
                };
                let next = move_toward(position, target, speed * dt);
                self.animals[index].position = [
                    next[0].clamp(-18.5, 18.5),
                    next[1],
                    next[2].clamp(-18.5, 18.5),
                ];
                let yaw = (target[0] - position[0]).atan2(-(target[2] - position[2]));
                let bob = if kind == AnimalKind::Hawk {
                    (self.elapsed * 2.0 + index as f32).sin() * 0.18
                } else {
                    0.0
                };
                self.animals[index].position[1] += bob * dt;
                scene.set_local_transform(
                    self.animals[index].entity,
                    Transform::default()
                        .with_translation(self.animals[index].position)
                        .with_rotation_y(yaw)
                        .with_scale(if self.animals[index].adult { 1.0 } else { 0.55 }),
                );
            }
        }
        for (target, damage) in attacks {
            self.damage_animal(scene, target, damage);
        }
        if player_damage > 0.0 {
            self.health = (self.health - player_damage).max(0.0);
            self.status = format!(
                "A wild wolf mauled you for {player_damage:.0}! Tamed wolves and hawks can defend you."
            );
            true
        } else {
            false
        }
    }

    fn update_viewmodels(&self, scene: &mut Scene, player: PlayerSnapshot) {
        for entity in self.viewmodels {
            scene.set_local_transform(entity, hidden_transform());
        }
        let forward = direction(player);
        let flat_forward = [player.yaw.sin(), 0.0, -player.yaw.cos()];
        let right = [player.yaw.cos(), 0.0, player.yaw.sin()];
        if player.camera_mode == CameraMode::FirstPerson {
            scene.set_local_transform(self.avatar, hidden_transform());
            if !self.build_mode {
                let position = add(
                    add(
                        add(player.position, scale(forward, 0.88)),
                        scale(right, 0.34),
                    ),
                    [0.0, -0.3, 0.0],
                );
                scene.set_local_transform(
                    self.viewmodels[self.selected.slot() as usize - 1],
                    Transform::default()
                        .with_translation(position)
                        .with_rotation_y(player.yaw)
                        .with_scale(0.45),
                );
            }
        } else {
            scene.set_local_transform(
                self.avatar,
                Transform::default()
                    .with_translation(add(player.position, [0.0, -0.91, 0.0]))
                    .with_rotation_y(player.yaw + PI),
            );
            if !self.build_mode {
                let position = add(
                    add(player.position, scale(flat_forward, 0.35)),
                    add(scale(right, 0.48), [0.0, -0.25, 0.0]),
                );
                scene.set_local_transform(
                    self.viewmodels[self.selected.slot() as usize - 1],
                    Transform::default()
                        .with_translation(position)
                        .with_rotation_y(player.yaw)
                        .with_scale(0.65),
                );
            }
        }
    }

    fn prompt(&self) -> String {
        if self.build_mode {
            let (wood, stone) = self.build_kind.cost();
            return format!(
                "BUILD {}  •  {wood} wood + {stone} stone  •  LMB place  •  X cycle  •  B exit",
                self.build_kind.name().to_uppercase()
            );
        }
        if matches!(self.selected, Tool::Pickaxe | Tool::Axe)
            && let Some(index) = self.aimed_resource(
                self.last_player,
                4.2,
                Some(if self.selected == Tool::Pickaxe {
                    NodeKind::Ore
                } else {
                    NodeKind::Tree
                }),
            )
        {
            let node = &self.resources[index];
            return format!(
                "LMB {}  •  {} hits left",
                if node.kind == NodeKind::Ore {
                    "Mine ore"
                } else {
                    "Cut tree"
                },
                node.hits
            );
        }
        if let Some(index) = self.aimed_animal(self.last_player, 4.5, 0.3) {
            let animal = &self.animals[index];
            let state = if animal.tamed {
                if animal.adult {
                    "tamed • E breed with nearby pair"
                } else {
                    "tamed youngster • growing"
                }
            } else {
                "wild"
            };
            return format!(
                "{} • {} • {state}  •  E interact ({} food)",
                animal.kind.name(),
                animal.sex.name(),
                animal.kind.food()
            );
        }
        if self
            .aimed_resource(self.last_player, 3.8, Some(NodeKind::Berry))
            .is_some()
        {
            return "Berry bush  •  E forage +4 berries".into();
        }
        match self.selected {
            Tool::Rifle => "Long-range precision  •  one round  •  R reload".into(),
            Tool::Pistol => "Fast close-range sidearm  •  six rounds  •  R reload".into(),
            Tool::Pickaxe => "Find a teal-veined rock and use LMB to mine".into(),
            Tool::Axe => "Find a tree and use LMB to cut wood".into(),
        }
    }
}

impl GamePlugin for OrnithGame {
    fn on_action(
        &mut self,
        action: GameAction,
        scene: &mut Scene,
        player: PlayerSnapshot,
    ) -> GameFeedback {
        self.last_player = player;
        match action {
            GameAction::PrimaryUse if self.build_mode => self.place_build(scene, player),
            GameAction::PrimaryUse if matches!(self.selected, Tool::Rifle | Tool::Pistol) => {
                self.fire(scene, player)
            }
            GameAction::PrimaryUse => self.harvest(scene, player),
            GameAction::Reload => self.reload(),
            GameAction::Interact => self.interact(scene, player),
            GameAction::SelectSlot(slot) => {
                if let Some(tool) = match slot {
                    1 => Some(Tool::Rifle),
                    2 => Some(Tool::Pistol),
                    3 => Some(Tool::Pickaxe),
                    4 => Some(Tool::Axe),
                    _ => None,
                } {
                    self.selected = tool;
                    self.build_mode = false;
                    self.reloading = None;
                    self.status = format!("{} equipped.", tool.name());
                }
                GameFeedback::default()
            }
            GameAction::ToggleBuild => {
                self.build_mode = !self.build_mode;
                self.reloading = None;
                self.status = if self.build_mode {
                    "Build mode: X cycles pieces, LMB places.".into()
                } else {
                    "Build mode closed.".into()
                };
                GameFeedback::default()
            }
            GameAction::CycleBuild => {
                self.build_kind = match self.build_kind {
                    BuildKind::Wall => BuildKind::Foundation,
                    BuildKind::Foundation => BuildKind::Wall,
                };
                self.status = format!("{} selected for building.", self.build_kind.name());
                GameFeedback::default()
            }
            GameAction::Eat => self.eat(),
            GameAction::CommandCompanions => {
                self.companions_follow = !self.companions_follow;
                for animal in &mut self.animals {
                    if animal.tamed {
                        animal.follow = self.companions_follow;
                    }
                }
                self.status = if self.companions_follow {
                    "Companions: follow me.".into()
                } else {
                    "Companions: stay here.".into()
                };
                GameFeedback::default()
            }
        }
    }

    fn fixed_update(
        &mut self,
        scene: &mut Scene,
        elapsed: Duration,
        player: PlayerSnapshot,
    ) -> GameFeedback {
        let dt = elapsed.as_secs_f32().min(0.1);
        self.last_player = player;
        self.elapsed += dt;
        self.intro_seconds = (self.intro_seconds - dt).max(0.0);
        self.action_cooldown = (self.action_cooldown - dt).max(0.0);
        if let Some((tool, remaining)) = self.reloading {
            let remaining = remaining - dt;
            if remaining <= 0.0 {
                *self.loaded_mut(tool).unwrap() = weapon_spec(tool).unwrap().magazine;
                self.reloading = None;
                self.status = format!("{} loaded and ready.", tool.name());
            } else {
                self.reloading = Some((tool, remaining));
            }
        }
        self.hunger = (self.hunger - dt * 0.12).max(0.0);
        if self.hunger <= 0.0 {
            self.health = (self.health - dt * 2.0).max(0.0);
        }
        let hurt = self.update_animals(scene, dt, player);
        self.update_viewmodels(scene, player);
        let mut feedback = GameFeedback::default();
        if hurt {
            feedback.sound_effects.push("hurt".into());
        }
        if self.health <= 0.0 || player.position[1] < -6.0 {
            self.health = 100.0;
            self.hunger = 100.0;
            self.rifle_loaded = 1;
            self.pistol_loaded = 6;
            self.reloading = None;
            self.status =
                "You washed ashore again. Your gathered supplies were kept for this test slice."
                    .into();
            feedback.respawn = Some(RESPAWN);
        }
        feedback
    }

    fn hud(&self) -> GameHud {
        let objective = if self.inventory.wood < 12 || self.inventory.stone < 6 {
            format!(
                "FIRST SHELTER\nWood {}/12  •  Stone {}/6",
                self.inventory.wood.min(12),
                self.inventory.stone.min(6)
            )
        } else if self.built_count == 0 {
            "FIRST SHELTER\nPress B and place your first structure".into()
        } else if !self
            .animals
            .iter()
            .any(|animal| animal.alive && animal.tamed)
        {
            "ISLAND BOND\nTame with meat (wolf/hawk) or berries (cow)".into()
        } else {
            "ORNITH OUTPOST\nHunt, breed companions, and expand".into()
        };
        let direction_name = match (self.last_player.yaw.rem_euclid(2.0 * PI) * 8.0 / (2.0 * PI))
            .round() as u8
            % 8
        {
            0 => "N",
            1 => "NE",
            2 => "E",
            3 => "SE",
            4 => "S",
            5 => "SW",
            6 => "W",
            _ => "NW",
        };
        let ammo = if let Some((tool, remaining)) = self.reloading {
            format!(
                "RELOADING {}  •  {:.1}s",
                tool.name().to_uppercase(),
                remaining.max(0.0)
            )
        } else if let Some(spec) = weapon_spec(self.selected) {
            format!(
                "{} / {}  •  {:.0}m effective",
                self.loaded(self.selected),
                spec.magazine,
                spec.range
            )
        } else {
            String::new()
        };
        GameHud {
            title: "SURVIVING ORNITH".into(),
            subtitle: "Ornith Island  •  UBE nerd 0.1".into(),
            objective,
            status: self.status.clone(),
            prompt: self.prompt(),
            resources: format!(
                "WOOD  {:02}     STONE  {:02}\nMEAT   {:02}     BERRIES {:02}\nSTRUCTURES  {:02}",
                self.inventory.wood,
                self.inventory.stone,
                self.inventory.meat,
                self.inventory.berries,
                self.built_count
            ),
            ammo,
            compass: format!(
                "{direction_name}  •  X {:.0}  Z {:.0}",
                self.last_player.position[0], self.last_player.position[2]
            ),
            controls: "WASD move  •  RMB look  •  V view  •  E interact  •  F eat  •  C companions"
                .into(),
            health: self.health,
            hunger: self.hunger,
            slots: [Tool::Rifle, Tool::Pistol, Tool::Pickaxe, Tool::Axe]
                .into_iter()
                .map(|tool| {
                    let detail = if let Some(spec) = weapon_spec(tool) {
                        format!("{} / {}", self.loaded(tool), spec.magazine)
                    } else {
                        "HARVEST".into()
                    };
                    HudSlot {
                        label: format!("{}  {}", tool.slot(), tool.name().to_uppercase()),
                        detail,
                        selected: !self.build_mode && self.selected == tool,
                    }
                })
                .collect(),
            crosshair: self.last_player.camera_mode != CameraMode::TopDown,
            intro_seconds: self.intro_seconds,
        }
    }
}

fn direction(player: PlayerSnapshot) -> [f32; 3] {
    [
        player.yaw.sin() * player.pitch.cos(),
        player.pitch.sin(),
        -player.yaw.cos() * player.pitch.cos(),
    ]
}

fn aim_score(
    player: PlayerSnapshot,
    target: [f32; 3],
    range: f32,
    minimum_dot: f32,
) -> Option<f32> {
    let offset = subtract(target, player.position);
    let distance = length(offset);
    if distance <= 0.001 || distance > range {
        return None;
    }
    let alignment = dot(direction(player), scale(offset, distance.recip()));
    (alignment >= minimum_dot).then_some(distance * (1.0 + (1.0 - alignment) * 3.0))
}

fn move_toward(position: [f32; 3], target: [f32; 3], amount: f32) -> [f32; 3] {
    let offset = subtract(target, position);
    let distance = length(offset);
    if distance <= amount || distance < 0.0001 {
        target
    } else {
        add(position, scale(offset, amount / distance))
    }
}

fn add(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}
fn subtract(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}
fn scale(value: [f32; 3], factor: f32) -> [f32; 3] {
    value.map(|component| component * factor)
}
fn dot(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}
fn length(value: [f32; 3]) -> f32 {
    dot(value, value).sqrt()
}
fn distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    length(subtract(left, right))
}
fn horizontal_distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    ((left[0] - right[0]).powi(2) + (left[2] - right[2]).powi(2)).sqrt()
}
fn interpolate(left: [f32; 3], right: [f32; 3], amount: f32) -> [f32; 3] {
    add(left, scale(subtract(right, left), amount))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(game: &mut OrnithGame, scene: &mut Scene, seconds: f32) -> GameFeedback {
        game.fixed_update(
            scene,
            Duration::from_secs_f32(seconds),
            PlayerSnapshot {
                position: RESPAWN,
                yaw: 0.0,
                pitch: 0.0,
                camera_mode: CameraMode::FirstPerson,
            },
        )
    }

    fn aiming_at(target: [f32; 3]) -> PlayerSnapshot {
        PlayerSnapshot {
            position: add(target, [0.0, 0.91, 3.2]),
            yaw: 0.0,
            pitch: 0.0,
            camera_mode: CameraMode::FirstPerson,
        }
    }

    #[test]
    fn rifle_requires_slow_reload_and_pistol_has_distinct_handling() {
        let (mut scene, mut game) = new_world();
        game.fire(
            &mut scene,
            PlayerSnapshot {
                position: RESPAWN,
                yaw: 0.0,
                pitch: 0.0,
                camera_mode: CameraMode::FirstPerson,
            },
        );
        assert_eq!(game.rifle_loaded, 0);
        game.reload();
        step(&mut game, &mut scene, 0.1);
        for _ in 0..24 {
            step(&mut game, &mut scene, 0.1);
        }
        assert_eq!(game.rifle_loaded, 1);
        game.on_action(GameAction::SelectSlot(2), &mut scene, game.last_player);
        assert_eq!(weapon_spec(game.selected).unwrap().spread, 0.055);
        assert!(weapon_spec(Tool::Rifle).unwrap().range > weapon_spec(Tool::Pistol).unwrap().range);
        assert!(
            weapon_spec(Tool::Rifle).unwrap().reload_seconds
                > weapon_spec(Tool::Pistol).unwrap().reload_seconds
        );
    }

    #[test]
    fn axe_yields_wood_and_removes_tree_collision() {
        let (mut scene, mut game) = new_world();
        let target = game.resources[0].position;
        let player = aiming_at(target);
        game.on_action(GameAction::SelectSlot(4), &mut scene, player);
        for _ in 0..3 {
            game.on_action(GameAction::PrimaryUse, &mut scene, player);
            game.action_cooldown = 0.0;
        }
        assert_eq!(game.inventory.wood, 6);
        assert!(!game.resources[0].alive);
        assert!(scene.collider(game.resources[0].entity).is_none());
    }

    #[test]
    fn pickaxe_yields_stone_from_ore() {
        let (mut scene, mut game) = new_world();
        let ore = game
            .resources
            .iter()
            .position(|node| node.kind == NodeKind::Ore)
            .unwrap();
        let player = aiming_at(game.resources[ore].position);
        game.on_action(GameAction::SelectSlot(3), &mut scene, player);
        for _ in 0..3 {
            game.on_action(GameAction::PrimaryUse, &mut scene, player);
            game.action_cooldown = 0.0;
        }
        assert_eq!(game.inventory.stone, 4);
        assert!(!game.resources[ore].alive);
    }

    #[test]
    fn building_spends_resources_and_activates_pooled_collision() {
        let (mut scene, mut game) = new_world();
        game.inventory.wood = 12;
        game.inventory.stone = 6;
        let player = game.last_player;
        game.on_action(GameAction::ToggleBuild, &mut scene, player);
        let feedback = game.on_action(GameAction::PrimaryUse, &mut scene, player);
        assert!(feedback.physics_changed);
        assert_eq!((game.inventory.wood, game.inventory.stone), (8, 4));
        assert!(game.builds[0].active);
        assert!(scene.collider(game.builds[0].entity).is_some());
    }

    #[test]
    fn building_rejects_player_overlap() {
        let (mut scene, mut game) = new_world();
        game.inventory.wood = 20;
        game.inventory.stone = 20;
        let player = PlayerSnapshot {
            position: [0.0, 0.91, 1.9],
            ..game.last_player
        };
        game.build_mode = true;
        game.on_action(GameAction::PrimaryUse, &mut scene, player);
        assert!(!game.builds[0].active);
        assert_eq!(game.inventory.wood, 20);
    }

    #[test]
    fn taming_consumes_species_appropriate_food() {
        let (mut scene, mut game) = new_world();
        let player = aiming_at(game.animals[0].position);
        let meat = game.inventory.meat;
        game.on_action(GameAction::Interact, &mut scene, player);
        assert!(game.animals[0].tamed);
        assert_eq!(game.inventory.meat, meat - 1);
        let cow = game
            .animals
            .iter()
            .position(|animal| animal.kind == AnimalKind::Cow)
            .unwrap();
        let player = aiming_at(game.animals[cow].position);
        let berries = game.inventory.berries;
        game.on_action(GameAction::Interact, &mut scene, player);
        assert!(game.animals[cow].tamed);
        assert_eq!(game.inventory.berries, berries - 1);
    }

    #[test]
    fn opposite_sex_tamed_pair_produces_live_offspring() {
        let (mut scene, mut game) = new_world();
        game.animals[0].tamed = true;
        game.animals[1].tamed = true;
        game.animals[1].position = game.animals[0].position;
        let player = aiming_at(game.animals[0].position);
        let before = game.animals.iter().filter(|animal| animal.alive).count();
        let meat = game.inventory.meat;
        game.on_action(GameAction::Interact, &mut scene, player);
        assert_eq!(
            game.animals.iter().filter(|animal| animal.alive).count(),
            before + 1
        );
        assert_eq!(game.inventory.meat, meat - 2);
        assert!(
            game.animals
                .iter()
                .any(|animal| animal.alive && !animal.adult)
        );
        assert!(game.animals[0].breed_cooldown > 0.0);
    }

    #[test]
    fn tamed_wolf_assists_against_hostile_wolf() {
        let (mut scene, mut game) = new_world();
        game.animals[0].tamed = true;
        game.animals[0].position = [7.0, 0.0, -8.0];
        game.animals[1].position = [8.0, 0.0, -8.0];
        let health = game.animals[1].health;
        step(&mut game, &mut scene, 0.1);
        assert!(game.animals[1].health < health);
    }

    #[test]
    fn eating_restores_hunger_and_death_respawns() {
        let (mut scene, mut game) = new_world();
        game.hunger = 40.0;
        game.eat();
        assert_eq!(game.hunger, 75.0);
        game.health = 0.0;
        let feedback = step(&mut game, &mut scene, 0.1);
        assert_eq!(feedback.respawn, Some(RESPAWN));
        assert_eq!(game.health, 100.0);
    }

    #[test]
    fn pistol_cannot_hit_beyond_its_range() {
        let (mut scene, mut game) = new_world();
        game.animals[0].position = [0.0, 0.0, -24.0];
        let player = PlayerSnapshot {
            position: RESPAWN,
            yaw: 0.0,
            pitch: 0.0,
            camera_mode: CameraMode::FirstPerson,
        };
        game.on_action(GameAction::SelectSlot(2), &mut scene, player);
        let health = game.animals[0].health;
        game.fire(&mut scene, player);
        assert_eq!(game.animals[0].health, health);
        assert_eq!(game.pistol_loaded, 5);
    }
}
