mod camera_controller;
mod camera_mode;
mod config;
mod debug;
mod error;
mod movement_audio;
mod simulation;

pub use camera_mode::CameraMode;
pub use config::AppConfig;
pub use engine_core::{CARGO_PACKAGE_VERSION, ENGINE_NAME, ENGINE_SHORT_NAME, RELEASE_VERSION};
pub use engine_scene::{
    BoxCollider, ColliderError, Entity, ModelReference, Scene, SceneDataError, SceneDocument,
    SceneEntity, SceneError, SceneFileError, Transform, load_scene, save_scene,
};
pub use error::RuntimeError;

use std::collections::HashMap;
use std::fs;
use std::time::Instant;

use camera_controller::CameraController;
use debug::{DebugCommand, DebugInspector};
use engine_assets::load_static_gltf;
use engine_audio::{AudioClip, AudioSystem};
use engine_core::FrameStats;
use engine_platform::{EventHandler, LoopControl, PlatformEvent, WindowConfig};
use engine_render::Renderer;
use movement_audio::FootstepCadence;
use simulation::Simulation;
use tracing::{info, warn};

/// Runs an application until its native event loop exits.
pub fn run(config: AppConfig) -> Result<(), RuntimeError> {
    info!(
        engine = ENGINE_NAME,
        short_name = ENGINE_SHORT_NAME,
        release = RELEASE_VERSION,
        title = config.title(),
        camera_mode = ?config.camera_mode(),
        "starting engine runtime"
    );

    let mut application = EngineApplication::new(config);
    let platform_result = engine_platform::run_event_loop(
        WindowConfig::new(application.config.title()),
        &mut application,
    );

    if let Some(error) = application.error {
        info!(error = %error, "engine runtime stopped with an error");
        return Err(error);
    }

    match platform_result {
        Ok(()) => {
            info!("engine runtime stopped");
            Ok(())
        }
        Err(error) => {
            let error = RuntimeError::platform(error);
            info!(error = %error, "engine runtime stopped with an error");
            Err(error)
        }
    }
}

struct EngineApplication {
    config: AppConfig,
    window: Option<engine_platform::NativeWindow>,
    renderer: Option<Renderer>,
    camera_controller: CameraController,
    simulation: Option<Simulation>,
    physics: Option<engine_physics::PhysicsWorld>,
    audio: Option<AudioSystem>,
    footstep_clip: Option<AudioClip>,
    footsteps: FootstepCadence,
    debug: DebugInspector,
    model_indices: HashMap<String, usize>,
    object_capacity: usize,
    last_frame: Option<Instant>,
    frame_stats: FrameStats,
    error: Option<RuntimeError>,
}

impl EngineApplication {
    fn new(config: AppConfig) -> Self {
        let camera_controller = CameraController::new(config.camera_mode());
        Self {
            config,
            window: None,
            renderer: None,
            camera_controller,
            simulation: None,
            physics: None,
            audio: None,
            footstep_clip: None,
            footsteps: FootstepCadence::default(),
            debug: DebugInspector::default(),
            model_indices: HashMap::new(),
            object_capacity: 0,
            last_frame: None,
            frame_stats: FrameStats::default(),
            error: None,
        }
    }

    fn fail(&mut self, error: RuntimeError) -> LoopControl {
        self.error = Some(error);
        LoopControl::Exit
    }

    fn render_frame(&mut self) -> Result<(), engine_render::RenderError> {
        let now = Instant::now();
        if let Some(last_frame) = self.last_frame.replace(now) {
            let elapsed = now.duration_since(last_frame);
            let physics = self
                .physics
                .as_mut()
                .expect("rendering requires initialized physics");
            let camera = &mut self.camera_controller;
            let audio = &self.audio;
            let footstep_clip = &self.footstep_clip;
            let footsteps = &mut self.footsteps;
            let simulation_outcome = self
                .simulation
                .as_mut()
                .expect("rendering requires an initialized simulation")
                .advance_with(elapsed, |step| {
                    let previous = camera.position();
                    let movement = camera.update(step, physics);
                    let horizontal_change = [
                        movement.position[0] - previous[0],
                        movement.position[2] - previous[2],
                    ];
                    let moved = horizontal_change
                        .iter()
                        .map(|value| value * value)
                        .sum::<f32>()
                        > 0.00000001;
                    if footsteps.update(movement.grounded && moved, step)
                        && let (Some(audio), Some(clip)) = (audio.as_ref(), footstep_clip.as_ref())
                    {
                        audio.play(clip);
                        info!(position = ?movement.position, "footstep playback requested");
                    }
                });
            if !simulation_outcome.dropped.is_zero() {
                tracing::debug!(
                    dropped_ms = simulation_outcome.dropped.as_secs_f64() * 1_000.0,
                    "simulation catch-up steps dropped"
                );
            }
            if let Some(statistics) = self.frame_stats.record(elapsed) {
                info!(
                    frames = statistics.frames,
                    fps = statistics.frames_per_second,
                    average_frame_ms = statistics.average_frame_ms,
                    maximum_frame_ms = statistics.maximum_frame_ms,
                    "frame statistics"
                );
            }
        }

        let debug_output = if self.debug.visible() {
            let size = self
                .window
                .as_ref()
                .expect("rendering requires a native window")
                .size();
            self.debug.frame(
                self.simulation
                    .as_ref()
                    .expect("rendering requires an initialized simulation")
                    .scene(),
                size.width,
                size.height,
            )
        } else {
            None
        };
        if let Some(renderer) = &mut self.renderer {
            let items = self
                .simulation
                .as_ref()
                .expect("rendering requires an initialized simulation")
                .model_transforms()
                .into_iter()
                .map(|item| engine_render::RenderItem {
                    model_index: self.model_indices[item.model_path],
                    transform: item.transform,
                })
                .collect::<Vec<_>>();
            renderer.render(self.camera_controller.camera(), &items, debug_output)?;
        }
        Ok(())
    }

    fn execute_debug_command(&mut self, command: DebugCommand) -> LoopControl {
        match command {
            DebugCommand::Toggle => {
                let visible = self.debug.toggle();
                self.camera_controller.clear_input();
                if let Some(window) = &self.window
                    && let Err(error) = window.set_cursor_captured(false)
                {
                    return self.fail(RuntimeError::platform(error));
                }
                info!(visible, "debug inspector visibility changed");
            }
            DebugCommand::SelectNext => {
                let selected = self.debug.select_next(
                    self.simulation
                        .as_ref()
                        .expect("debugging requires an initialized simulation")
                        .scene(),
                );
                info!(?selected, "debug entity selection changed");
            }
            DebugCommand::Edit(edit) => {
                let result = self.debug.apply_edit(
                    self.simulation
                        .as_mut()
                        .expect("debugging requires an initialized simulation")
                        .scene_mut(),
                    edit,
                );
                match result {
                    Ok((entity, transform)) => {
                        self.physics = Some(engine_physics::PhysicsWorld::from_scene(
                            self.simulation.as_ref().unwrap().scene(),
                            self.camera_controller.position(),
                        ));
                        info!(
                            entity = entity.index(),
                            generation = entity.generation(),
                            ?transform,
                            "debug entity edited"
                        );
                    }
                    Err(error) => {
                        warn!(%error, "debug entity edit failed");
                        self.debug.set_status(error);
                    }
                }
            }
            DebugCommand::Save => {
                let path = self.config.workspace_scene_path().to_path_buf();
                let result = (|| {
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent).map_err(|error| {
                            format!(
                                "failed to create scene directory {}: {error}",
                                parent.display()
                            )
                        })?;
                    }
                    save_scene(&path, self.simulation.as_ref().unwrap().scene())
                        .map_err(|error| error.to_string())
                })();
                match result {
                    Ok(()) => {
                        info!(path = %path.display(), "debug scene saved");
                        self.debug.set_status(format!("Saved {}", path.display()));
                    }
                    Err(error) => {
                        warn!(path = %path.display(), %error, "debug scene save failed");
                        self.debug.set_status(error);
                    }
                }
            }
            DebugCommand::Load => {
                let path = self.config.workspace_scene_path().to_path_buf();
                let scene = match load_scene(&path) {
                    Ok(scene) => scene,
                    Err(error) => {
                        let message = error.to_string();
                        warn!(path = %path.display(), error = %message, "debug scene load failed");
                        self.debug.set_status(message);
                        return LoopControl::Continue;
                    }
                };
                let replacement = match Simulation::from_scene(scene, None) {
                    Ok(simulation) => simulation,
                    Err(error) => {
                        let message = error.to_string();
                        warn!(path = %path.display(), error = %message, "debug scene load failed");
                        self.debug.set_status(message);
                        return LoopControl::Continue;
                    }
                };
                let models = replacement.model_transforms();
                let unsupported_model = models
                    .iter()
                    .find(|item| !self.model_indices.contains_key(item.model_path))
                    .map(|item| item.model_path.to_owned());
                if models.len() > self.object_capacity || unsupported_model.is_some() {
                    let message = format!(
                        "reload requires already uploaded models and at most {} objects; unsupported model: {}",
                        self.object_capacity,
                        unsupported_model.as_deref().unwrap_or("none")
                    );
                    warn!(path = %path.display(), error = %message, "debug scene load failed");
                    self.debug.set_status(message);
                    return LoopControl::Continue;
                }
                let physics = engine_physics::PhysicsWorld::from_scene(
                    replacement.scene(),
                    self.camera_controller.position(),
                );
                self.simulation = Some(replacement);
                self.physics = Some(physics);
                self.footsteps = FootstepCadence::default();
                self.debug.clear_selection();
                self.debug.set_status(format!("Loaded {}", path.display()));
                info!(path = %path.display(), "debug scene loaded");
            }
        }
        LoopControl::Continue
    }
}

impl EventHandler for EngineApplication {
    fn on_event(&mut self, event: PlatformEvent<'_>) -> LoopControl {
        if self.error.is_some() {
            return LoopControl::Exit;
        }

        match event {
            PlatformEvent::Resumed(window) => {
                self.window = Some(window.clone());
                if self.renderer.is_some() {
                    return LoopControl::Continue;
                }

                let scene = if let Some(scene) = self.config.take_initial_scene() {
                    scene
                } else {
                    match load_scene(self.config.scene_path()) {
                        Ok(scene) => scene,
                        Err(error) => return self.fail(RuntimeError::scene(error)),
                    }
                };
                let physics = engine_physics::PhysicsWorld::from_scene(
                    &scene,
                    self.camera_controller.position(),
                );
                let scene_update = self.config.take_scene_update();
                let simulation = match Simulation::from_scene(scene, scene_update) {
                    Ok(simulation) => simulation,
                    Err(error) => return self.fail(RuntimeError::simulation(error)),
                };
                let mut assets = Vec::new();
                for item in simulation.model_transforms() {
                    if self.model_indices.contains_key(item.model_path) {
                        continue;
                    }
                    let model = match load_static_gltf(item.model_path) {
                        Ok(model) => model,
                        Err(error) => return self.fail(RuntimeError::asset(error)),
                    };
                    self.model_indices
                        .insert(item.model_path.to_owned(), assets.len());
                    assets.push(model);
                }
                let (audio, footstep_clip) = if let Some(path) = self.config.footstep_sound_path() {
                    let clip = match AudioClip::load(path) {
                        Ok(clip) => clip,
                        Err(error) => return self.fail(RuntimeError::audio(error)),
                    };
                    let audio = match AudioSystem::new() {
                        Ok(audio) => audio,
                        Err(error) => return self.fail(RuntimeError::audio(error)),
                    };
                    let output = audio.output_info();
                    info!(
                        channels = output.channels,
                        sample_rate = output.sample_rate,
                        clip_frames = clip.frames(),
                        clip_duration_ms = clip.duration().as_secs_f64() * 1_000.0,
                        "audio initialized"
                    );
                    (Some(audio), Some(clip))
                } else {
                    info!("audio disabled");
                    (None, None)
                };
                let object_capacity = simulation.model_transforms().len();
                self.object_capacity = object_capacity;
                self.audio = audio;
                self.footstep_clip = footstep_clip;
                self.physics = Some(physics);
                self.simulation = Some(simulation);
                match Renderer::new(window.clone(), &assets, object_capacity) {
                    Ok(renderer) => {
                        self.renderer = Some(renderer);
                        LoopControl::Continue
                    }
                    Err(error) => self.fail(RuntimeError::render(error)),
                }
            }
            PlatformEvent::CloseRequested => LoopControl::Exit,
            PlatformEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size);
                }
                LoopControl::Continue
            }
            PlatformEvent::KeyboardInput(input) => {
                if let Some(command) = self.debug.command(input) {
                    return self.execute_debug_command(command);
                }
                if input.key == engine_platform::Key::V {
                    if input.pressed {
                        let mode = self.camera_controller.cycle_camera_mode();
                        info!(
                            ?mode,
                            position = ?self.camera_controller.camera().position(),
                            target = ?self.camera_controller.camera().target(),
                            "camera mode changed"
                        );
                    }
                } else if input.key == engine_platform::Key::Escape && input.pressed {
                    self.camera_controller.clear_input();
                    if let Some(window) = &self.window
                        && let Err(error) = window.set_cursor_captured(false)
                    {
                        return self.fail(RuntimeError::platform(error));
                    }
                } else {
                    self.camera_controller.handle_key(input);
                    if !input.pressed {
                        info!(
                            position = ?self.camera_controller.position(),
                            "character position"
                        );
                    }
                }
                LoopControl::Continue
            }
            PlatformEvent::MouseLookButton { pressed } => {
                info!(pressed, "mouse look button changed");
                if pressed {
                    self.camera_controller.set_mouse_look(true);
                    if let Some(window) = &self.window
                        && let Err(error) = window.set_cursor_captured(true)
                    {
                        return self.fail(RuntimeError::platform(error));
                    }
                }
                LoopControl::Continue
            }
            PlatformEvent::MouseMotion { delta_x, delta_y } => {
                tracing::debug!(delta_x, delta_y, "mouse motion received");
                self.camera_controller.handle_mouse_motion(delta_x, delta_y);
                LoopControl::Continue
            }
            PlatformEvent::Focused(focused) => {
                info!(focused, "window focus changed");
                if !focused {
                    self.camera_controller.clear_input();
                    if let Some(window) = &self.window
                        && let Err(error) = window.set_cursor_captured(false)
                    {
                        return self.fail(RuntimeError::platform(error));
                    }
                }
                LoopControl::Continue
            }
            PlatformEvent::RedrawRequested => {
                if let Err(error) = self.render_frame() {
                    return self.fail(RuntimeError::render(error));
                }
                LoopControl::Continue
            }
            PlatformEvent::AboutToWait => {
                if let Some(renderer) = &self.renderer {
                    renderer.request_redraw();
                }
                LoopControl::Continue
            }
            PlatformEvent::Exiting => {
                self.renderer.take();
                self.footstep_clip.take();
                self.audio.take();
                LoopControl::Continue
            }
        }
    }
}
