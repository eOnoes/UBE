use engine_platform::{Key, KeyInput};
use engine_scene::{Entity, Scene, Transform};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DebugEdit {
    Translate([f32; 3]),
    RotateY(f32),
    MultiplyScale(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DebugCommand {
    Toggle,
    SelectNext,
    Edit(DebugEdit),
    Save,
    Load,
}

/// Runtime-owned selected-entity inspection state and immediate-mode overlay.
pub struct DebugInspector {
    context: egui::Context,
    visible: bool,
    selected: Option<Entity>,
    status: String,
}

impl Default for DebugInspector {
    fn default() -> Self {
        let context = egui::Context::default();
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "IBM Plex Mono".into(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../../../assets/fonts/IBMPlexMono-Regular.ttf"
            ))),
        );
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            fonts
                .families
                .entry(family)
                .or_default()
                .insert(0, "IBM Plex Mono".into());
        }
        context.set_fonts(fonts);
        Self {
            context,
            visible: false,
            selected: None,
            status: "F3 close | Tab select | arrows move | PgUp/PgDn height".into(),
        }
    }
}

impl DebugInspector {
    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn toggle(&mut self) -> bool {
        self.visible = !self.visible;
        self.visible
    }

    pub fn clear_selection(&mut self) {
        self.selected = None;
    }

    pub fn set_status(&mut self, status: impl Into<String>) {
        self.status = status.into();
    }

    pub fn command(&self, input: KeyInput) -> Option<DebugCommand> {
        if !input.pressed {
            return None;
        }
        if input.key == Key::F3 {
            return Some(DebugCommand::Toggle);
        }
        if !self.visible {
            return None;
        }
        match input.key {
            Key::Tab => Some(DebugCommand::SelectNext),
            Key::F5 => Some(DebugCommand::Save),
            Key::F9 => Some(DebugCommand::Load),
            Key::ArrowLeft => Some(DebugCommand::Edit(DebugEdit::Translate([-0.1, 0.0, 0.0]))),
            Key::ArrowRight => Some(DebugCommand::Edit(DebugEdit::Translate([0.1, 0.0, 0.0]))),
            Key::ArrowUp => Some(DebugCommand::Edit(DebugEdit::Translate([0.0, 0.0, -0.1]))),
            Key::ArrowDown => Some(DebugCommand::Edit(DebugEdit::Translate([0.0, 0.0, 0.1]))),
            Key::PageUp => Some(DebugCommand::Edit(DebugEdit::Translate([0.0, 0.1, 0.0]))),
            Key::PageDown => Some(DebugCommand::Edit(DebugEdit::Translate([0.0, -0.1, 0.0]))),
            Key::BracketLeft => Some(DebugCommand::Edit(DebugEdit::RotateY(
                -15.0_f32.to_radians(),
            ))),
            Key::BracketRight => Some(DebugCommand::Edit(DebugEdit::RotateY(
                15.0_f32.to_radians(),
            ))),
            Key::Minus => Some(DebugCommand::Edit(DebugEdit::MultiplyScale(1.0 / 1.1))),
            Key::Equal => Some(DebugCommand::Edit(DebugEdit::MultiplyScale(1.1))),
            _ => None,
        }
    }

    pub fn select_next(&mut self, scene: &Scene) -> Option<Entity> {
        let entities = scene.entities();
        if entities.is_empty() {
            self.selected = None;
            return None;
        }
        let current = self
            .selected
            .and_then(|selected| entities.iter().position(|entity| *entity == selected));
        self.selected = Some(entities[current.map_or(0, |index| (index + 1) % entities.len())]);
        self.selected
    }

    pub fn apply_edit(
        &mut self,
        scene: &mut Scene,
        edit: DebugEdit,
    ) -> Result<(Entity, Transform), String> {
        let entity = self
            .selected_entity(scene)
            .ok_or_else(|| "scene contains no entity to edit".to_owned())?;
        let current = scene
            .local_transform(entity)
            .ok_or_else(|| "selected entity no longer exists".to_owned())?;
        let transform = match edit {
            DebugEdit::Translate(offset) => Transform {
                translation: [
                    current.translation[0] + offset[0],
                    current.translation[1] + offset[1],
                    current.translation[2] + offset[2],
                ],
                ..current
            },
            DebugEdit::RotateY(radians) => current.rotated_y(radians),
            DebugEdit::MultiplyScale(factor) => {
                current.with_scale((current.scale * factor).clamp(0.1, 10.0))
            }
        };
        if !scene.set_local_transform(entity, transform) {
            return Err("selected entity could not be updated".into());
        }
        self.status = format!("Edited entity {}:{}", entity.index(), entity.generation());
        Ok((entity, transform))
    }

    pub fn frame(
        &mut self,
        scene: &Scene,
        width: u32,
        height: u32,
    ) -> Option<(egui::Context, egui::FullOutput)> {
        if !self.visible {
            return None;
        }
        let selected = self.selected_entity(scene)?;
        let transform = scene.local_transform(selected)?;
        let parent = scene.parent(selected);
        let model = scene.model(selected).map(|model| model.as_str().to_owned());
        let collider = scene.collider(selected);
        let status = self.status.clone();
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width.max(1) as f32, height.max(1) as f32),
            )),
            ..Default::default()
        };
        let output = self.context.run_pass(raw_input, |context| {
            egui::Window::new("Scene Debug Inspector")
                .default_pos(egui::pos2(12.0, 12.0))
                .default_width(410.0)
                .collapsible(false)
                .resizable(false)
                .show(context, |ui| {
                    ui.label(format!(
                        "Entity {}:{} | parent {}",
                        selected.index(),
                        selected.generation(),
                        parent
                            .map(|entity| format!("{}:{}", entity.index(), entity.generation()))
                            .unwrap_or_else(|| "none".into())
                    ));
                    ui.label(format!("Model: {}", model.as_deref().unwrap_or("none")));
                    ui.label(match collider {
                        Some(collider) => format!("Box collider: {:?}", collider.half_extents()),
                        None => "Box collider: none".into(),
                    });
                    ui.separator();
                    ui.monospace(format!("Translation: {:.3?}", transform.translation));
                    ui.monospace(format!("Rotation xyzw: {:.3?}", transform.rotation));
                    ui.monospace(format!("Uniform scale: {:.3}", transform.scale));
                    ui.separator();
                    ui.label("Tab select | arrows X/Z | PgUp/PgDn Y");
                    ui.label("[ ] rotate Y | - = scale | F5 save | F9 reload");
                    ui.label(&status);
                });
        });
        Some((self.context.clone(), output))
    }

    fn selected_entity(&mut self, scene: &Scene) -> Option<Entity> {
        if self.selected.is_some_and(|entity| scene.contains(entity)) {
            return self.selected;
        }
        let entities = scene.entities();
        self.selected = entities
            .iter()
            .copied()
            .find(|entity| scene.model(*entity).is_some())
            .or_else(|| entities.first().copied());
        self.selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_scene::{BoxCollider, ModelReference};

    fn scene() -> (Scene, Entity) {
        let mut scene = Scene::new();
        let root = scene.spawn(Transform::default());
        let model = scene.spawn(Transform::default().with_translation([1.0, 2.0, 3.0]));
        scene.set_parent(model, Some(root)).unwrap();
        scene.set_model(
            model,
            Some(ModelReference::new("assets/models/BoxTextured.glb").unwrap()),
        );
        scene.set_collider(model, Some(BoxCollider::new([0.5; 3]).unwrap()));
        (scene, model)
    }

    #[test]
    fn selection_prefers_models_and_cycles_through_live_entities() {
        let (scene, model) = scene();
        let mut inspector = DebugInspector::default();
        assert!(inspector.frame(&scene, 800, 600).is_none());
        inspector.toggle();
        let (_, mut output) = inspector.frame(&scene, 800, 600).unwrap();
        output.textures_delta.clear();
        assert_eq!(
            inspector
                .apply_edit(&mut scene.clone(), DebugEdit::RotateY(0.0))
                .unwrap()
                .0,
            model
        );
        assert_ne!(inspector.select_next(&scene), Some(model));
        assert_eq!(inspector.select_next(&scene), Some(model));
    }

    #[test]
    fn edits_update_translation_rotation_and_bounded_scale() {
        let (mut scene, model) = scene();
        let mut inspector = DebugInspector::default();
        let (_, translated) = inspector
            .apply_edit(&mut scene, DebugEdit::Translate([0.5, -1.0, 2.0]))
            .unwrap();
        assert_eq!(translated.translation, [1.5, 1.0, 5.0]);

        let (_, rotated) = inspector
            .apply_edit(&mut scene, DebugEdit::RotateY(90.0_f32.to_radians()))
            .unwrap();
        assert!((rotated.rotation[1] - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.0001);

        for _ in 0..100 {
            inspector
                .apply_edit(&mut scene, DebugEdit::MultiplyScale(1.1))
                .unwrap();
        }
        assert_eq!(scene.local_transform(model).unwrap().scale, 10.0);
    }

    #[test]
    fn commands_require_pressed_keys_and_a_visible_inspector() {
        let mut inspector = DebugInspector::default();
        let input = |key| KeyInput { key, pressed: true };
        assert_eq!(inspector.command(input(Key::ArrowRight)), None);
        assert_eq!(
            inspector.command(input(Key::F3)),
            Some(DebugCommand::Toggle)
        );
        inspector.toggle();
        assert_eq!(
            inspector.command(input(Key::ArrowRight)),
            Some(DebugCommand::Edit(DebugEdit::Translate([0.1, 0.0, 0.0])))
        );
        assert_eq!(inspector.command(input(Key::F5)), Some(DebugCommand::Save));
    }
}
