use std::error::Error;
use std::process::ExitCode;

use engine::{AppConfig, ENGINE_NAME, RELEASE_VERSION};

fn main() -> ExitCode {
    tracing_subscriber::fmt().with_target(false).init();
    let mut config = AppConfig::new(format!("{ENGINE_NAME} Sandbox — {RELEASE_VERSION}"));
    if let Some(scene_path) = std::env::args_os().nth(1) {
        config = config.with_scene_path(scene_path);
    }
    if let Some(audio_path) = std::env::args_os().nth(2) {
        config = config.with_footstep_sound_path(audio_path);
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
