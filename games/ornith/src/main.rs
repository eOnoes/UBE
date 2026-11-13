use std::error::Error;
use std::process::ExitCode;

use engine::{AppConfig, CameraMode};
use surviving_ornith::{SOUND_EFFECTS, new_world};

fn main() -> ExitCode {
    tracing_subscriber::fmt().with_target(false).init();
    let (scene, game) = new_world();
    let mut config = AppConfig::new("Surviving Ornith — UBE nerd 0.1")
        .with_scene(scene)
        .with_game_plugin(game)
        .with_camera_mode(CameraMode::FirstPerson)
        .with_workspace_scene_path("artifacts/runtime/ornith.scene.json")
        .with_footstep_sound_path("assets/ornith/audio/footstep.wav");
    for effect in SOUND_EFFECTS {
        config = config.with_sound_effect(*effect, format!("assets/ornith/audio/{effect}.wav"));
    }
    if std::env::args_os().any(|argument| argument == "--no-audio") {
        config = config.without_audio();
    }
    if let Err(error) = engine::run(config) {
        eprintln!("error: {error}");
        let mut source = error.source();
        while let Some(error) = source {
            eprintln!("caused by: {error}");
            source = error.source();
        }
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
