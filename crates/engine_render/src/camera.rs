use glam::Vec3;
use glam::camera::rh::proj::directx::perspective;
use glam::camera::rh::view::look_at_mat4;

/// Camera input consumed by the renderer for one frame.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    position: [f32; 3],
    target: [f32; 3],
    up: [f32; 3],
    vertical_fov_radians: f32,
    near_plane: f32,
    far_plane: f32,
}

impl Camera {
    pub fn new(position: [f32; 3], target: [f32; 3]) -> Self {
        Self {
            position,
            target,
            ..Default::default()
        }
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn target(&self) -> [f32; 3] {
        self.target
    }

    pub(crate) fn view_projection(&self, aspect_ratio: f32) -> [[f32; 4]; 4] {
        let view = look_at_mat4(
            Vec3::from(self.position),
            Vec3::from(self.target),
            Vec3::from(self.up),
        );
        let projection = perspective(
            self.vertical_fov_radians,
            aspect_ratio,
            self.near_plane,
            self.far_plane,
        );
        (projection * view).to_cols_array_2d()
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: [0.0, 0.91, 6.0],
            target: [0.0, 0.91, 0.0],
            up: [0.0, 1.0, 0.0],
            vertical_fov_radians: 45.0_f32.to_radians(),
            near_plane: 0.1,
            far_plane: 100.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_camera_produces_finite_view_projection() {
        let matrix = Camera::default().view_projection(16.0 / 9.0);

        assert!(matrix.iter().flatten().all(|value| value.is_finite()));
        assert_ne!(matrix, [[0.0; 4]; 4]);
    }

    #[test]
    fn aspect_ratio_changes_projection() {
        let camera = Camera::default();

        assert_ne!(camera.view_projection(1.0), camera.view_projection(2.0));
    }
}
