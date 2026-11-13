use std::time::Duration;

use engine_physics::PlayerStance;
use engine_scene::Scene;

use crate::CameraMode;

/// Engine-owned actions available to an application gameplay plugin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameAction {
    PrimaryUse,
    Reload,
    Interact,
    SelectSlot(u8),
    ToggleBuild,
    CycleBuild,
    Eat,
    CommandCompanions,
}

/// Read-only player/camera state supplied to gameplay without exposing native input types.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerSnapshot {
    pub position: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub camera_mode: CameraMode,
    pub stance: PlayerStance,
    pub sprint_energy: f32,
    pub sprinting: bool,
    pub grounded: bool,
}

/// Side effects requested by gameplay and applied by the engine at a safe boundary.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameFeedback {
    pub physics_changed: bool,
    pub respawn: Option<[f32; 3]>,
    pub sound_effects: Vec<String>,
}

/// Application-facing gameplay hooks. Native input and rendering stay behind the engine API.
pub trait GamePlugin: 'static {
    fn on_action(
        &mut self,
        _action: GameAction,
        _scene: &mut Scene,
        _player: PlayerSnapshot,
    ) -> GameFeedback {
        GameFeedback::default()
    }

    fn fixed_update(
        &mut self,
        _scene: &mut Scene,
        _elapsed: Duration,
        _player: PlayerSnapshot,
    ) -> GameFeedback {
        GameFeedback::default()
    }

    fn hud(&self) -> GameHud;
}

#[derive(Clone, Debug, PartialEq)]
pub struct HudSlot {
    pub label: String,
    pub detail: String,
    pub selected: bool,
}

/// Renderer-neutral game HUD data. Applications do not depend on egui.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameHud {
    pub title: String,
    pub subtitle: String,
    pub objective: String,
    pub status: String,
    pub prompt: String,
    pub resources: String,
    pub ammo: String,
    pub compass: String,
    pub controls: String,
    pub health: f32,
    pub hunger: f32,
    pub energy: Option<f32>,
    pub slots: Vec<HudSlot>,
    pub crosshair: bool,
    pub intro_seconds: f32,
}
