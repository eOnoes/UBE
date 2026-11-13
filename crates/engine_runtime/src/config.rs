use std::path::{Path, PathBuf};
use std::time::Duration;

use engine_scene::Scene;

use crate::{CameraMode, simulation::SceneUpdate};

/// Application-facing runtime configuration.
pub struct AppConfig {
    title: String,
    scene_path: PathBuf,
    initial_scene: Option<Scene>,
    scene_update: Option<SceneUpdate>,
    footstep_sound_path: Option<PathBuf>,
    workspace_scene_path: PathBuf,
    camera_mode: CameraMode,
}

impl AppConfig {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            scene_path: PathBuf::from("assets/scenes/orbit.scene.json"),
            initial_scene: None,
            scene_update: None,
            footstep_sound_path: Some(PathBuf::from("assets/audio/footstep.wav")),
            workspace_scene_path: PathBuf::from("artifacts/runtime/edited.scene.json"),
            camera_mode: CameraMode::default(),
        }
    }

    pub fn with_scene_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.scene_path = path.into();
        self.initial_scene = None;
        self
    }

    /// Starts with an application-built scene instead of reading the configured scene path.
    pub fn with_scene(mut self, scene: Scene) -> Self {
        self.initial_scene = Some(scene);
        self
    }

    /// Runs an application scene control after each fixed simulation step.
    pub fn with_scene_update(mut self, update: impl FnMut(&mut Scene, Duration) + 'static) -> Self {
        self.scene_update = Some(Box::new(update));
        self
    }

    /// Selects the WAV clip used for grounded character footsteps.
    pub fn with_footstep_sound_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.footstep_sound_path = Some(path.into());
        self
    }

    /// Disables audio output and gameplay footstep requests for this application.
    pub fn without_audio(mut self) -> Self {
        self.footstep_sound_path = None;
        self
    }

    /// Selects the scene file written by F5 and read by F9 in the debug inspector.
    pub fn with_workspace_scene_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.workspace_scene_path = path.into();
        self
    }

    /// Selects the initial gameplay camera. V cycles modes at runtime.
    pub fn with_camera_mode(mut self, mode: CameraMode) -> Self {
        self.camera_mode = mode;
        self
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn scene_path(&self) -> &Path {
        &self.scene_path
    }

    pub fn footstep_sound_path(&self) -> Option<&Path> {
        self.footstep_sound_path.as_deref()
    }

    pub fn workspace_scene_path(&self) -> &Path {
        &self.workspace_scene_path
    }

    pub fn camera_mode(&self) -> CameraMode {
        self.camera_mode
    }

    pub(crate) fn take_initial_scene(&mut self) -> Option<Scene> {
        self.initial_scene.take()
    }

    pub(crate) fn take_scene_update(&mut self) -> Option<SceneUpdate> {
        self.scene_update.take()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new(format!(
            "{} — {}",
            engine_core::ENGINE_NAME,
            engine_core::RELEASE_VERSION
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn application_can_supply_scene_and_fixed_step_update() {
        let scene = Scene::new();
        let mut config = AppConfig::new("Test")
            .with_scene(scene.clone())
            .with_scene_update(|_, _| {});

        assert_eq!(
            config.take_initial_scene().unwrap().to_document(),
            scene.to_document()
        );
        assert!(config.take_scene_update().is_some());
    }

    #[test]
    fn audio_path_is_configurable_and_can_be_disabled() {
        let config = AppConfig::new("Test").with_footstep_sound_path("assets/custom.wav");
        assert_eq!(
            config.footstep_sound_path(),
            Some(Path::new("assets/custom.wav"))
        );
        assert_eq!(config.without_audio().footstep_sound_path(), None);
    }

    #[test]
    fn public_identity_and_initial_camera_are_configurable() {
        let default = AppConfig::default();
        assert_eq!(
            default.title(),
            format!(
                "{} — {}",
                engine_core::ENGINE_NAME,
                engine_core::RELEASE_VERSION
            )
        );
        assert_eq!(default.camera_mode(), CameraMode::FirstPerson);
        assert_eq!(
            AppConfig::new("Test")
                .with_camera_mode(CameraMode::TopDown)
                .camera_mode(),
            CameraMode::TopDown
        );
    }

    #[test]
    fn scene_path_is_application_configurable() {
        let config = AppConfig::new("Test").with_scene_path("assets/example.scene.json");

        assert_eq!(config.scene_path(), Path::new("assets/example.scene.json"));
    }
}
