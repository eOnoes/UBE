use std::error::Error;

use crate::simulation::SimulationError;
use engine_assets::AssetError;
use engine_audio::AudioError;
use engine_platform::PlatformError;
use engine_render::RenderError;
use engine_scene::SceneFileError;
use thiserror::Error;

/// A fatal error that prevented the runtime from completing normally.
#[derive(Debug, Error)]
#[error("engine runtime failed")]
pub struct RuntimeError {
    #[source]
    source: Box<dyn Error + Send + Sync + 'static>,
}

impl RuntimeError {
    pub(crate) fn platform(error: PlatformError) -> Self {
        Self {
            source: Box::new(error),
        }
    }

    pub(crate) fn asset(error: AssetError) -> Self {
        Self {
            source: Box::new(error),
        }
    }

    pub(crate) fn scene(error: SceneFileError) -> Self {
        Self {
            source: Box::new(error),
        }
    }

    pub(crate) fn simulation(error: SimulationError) -> Self {
        Self {
            source: Box::new(error),
        }
    }

    pub(crate) fn audio(error: AudioError) -> Self {
        Self {
            source: Box::new(error),
        }
    }

    pub(crate) fn render(error: RenderError) -> Self {
        Self {
            source: Box::new(error),
        }
    }
}
