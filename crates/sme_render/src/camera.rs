//! 2D orthographic camera that produces a view-projection matrix for the sprite pipeline.
//!
//! The camera defines the visible world-space region as a centered rectangle
//! around `position`, scaled by `zoom`. The resulting `CameraUniform` is uploaded
//! to a GPU uniform buffer each frame and consumed by the sprite vertex shader.

use glam::{Mat4, Vec2};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

pub struct Camera2D {
    pub position: Vec2,
    pub zoom: f32,
    pub viewport: (u32, u32),
}

impl Camera2D {
    pub fn new(viewport_width: u32, viewport_height: u32) -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            viewport: (viewport_width, viewport_height),
        }
    }

    pub fn build_uniform(&self) -> CameraUniform {
        let half_w = (self.viewport.0 as f32) / (2.0 * self.zoom);
        let half_h = (self.viewport.1 as f32) / (2.0 * self.zoom);

        let proj = Mat4::orthographic_rh(
            self.position.x - half_w,
            self.position.x + half_w,
            self.position.y - half_h,
            self.position.y + half_h,
            -1.0,
            1.0,
        );

        CameraUniform {
            view_proj: proj.to_cols_array_2d(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Mat4, Vec2, Vec3};

    const TOLERANCE: f32 = 0.001;

    fn assert_approx(actual: f32, expected: f32, label: &str) {
        assert!(
            (actual - expected).abs() < TOLERANCE,
            "{label}: expected {expected}, got {actual}"
        );
    }

    fn proj_matrix(cam: &Camera2D) -> Mat4 {
        let uniform = cam.build_uniform();
        Mat4::from_cols_array_2d(&uniform.view_proj)
    }

    #[test]
    fn test_new_defaults() {
        let cam = Camera2D::new(800, 600);
        assert_eq!(cam.position, Vec2::ZERO);
        assert!((cam.zoom - 1.0).abs() < TOLERANCE, "zoom should be 1.0");
        assert_eq!(cam.viewport, (800, 600));
    }

    #[test]
    fn test_build_uniform_identity_center() {
        let cam = Camera2D::new(800, 600);
        let proj = proj_matrix(&cam);
        let result = proj.project_point3(Vec3::ZERO);
        assert_approx(result.x, 0.0, "center x");
        assert_approx(result.y, 0.0, "center y");
    }

    #[test]
    fn test_build_uniform_corners() {
        let cam = Camera2D::new(800, 600);
        let proj = proj_matrix(&cam);

        // Bottom-left corner
        let bl = proj.project_point3(Vec3::new(-400.0, -300.0, 0.0));
        assert_approx(bl.x, -1.0, "bottom-left x");
        assert_approx(bl.y, -1.0, "bottom-left y");

        // Top-right corner
        let tr = proj.project_point3(Vec3::new(400.0, 300.0, 0.0));
        assert_approx(tr.x, 1.0, "top-right x");
        assert_approx(tr.y, 1.0, "top-right y");
    }

    #[test]
    fn test_position_offset() {
        let mut cam = Camera2D::new(800, 600);
        cam.position = Vec2::new(100.0, 50.0);
        let proj = proj_matrix(&cam);

        // Camera center should map to clip origin
        let center = proj.project_point3(Vec3::new(100.0, 50.0, 0.0));
        assert_approx(center.x, 0.0, "offset center x");
        assert_approx(center.y, 0.0, "offset center y");

        // Bottom-left of visible area
        let bl = proj.project_point3(Vec3::new(-300.0, -250.0, 0.0));
        assert_approx(bl.x, -1.0, "offset bottom-left x");
        assert_approx(bl.y, -1.0, "offset bottom-left y");
    }

    #[test]
    fn test_zoom_in() {
        let mut cam = Camera2D::new(800, 600);
        cam.zoom = 2.0;
        let proj = proj_matrix(&cam);

        // At zoom=2, visible half-extents are 200x150
        let edge = proj.project_point3(Vec3::new(200.0, 150.0, 0.0));
        assert_approx(edge.x, 1.0, "zoom-in edge x");
        assert_approx(edge.y, 1.0, "zoom-in edge y");

        // Original viewport edge should now be outside clip space
        let outside = proj.project_point3(Vec3::new(400.0, 300.0, 0.0));
        assert!(
            outside.x > 1.0,
            "zoom-in: (400,300) x should be >1, got {}",
            outside.x
        );
        assert!(
            outside.y > 1.0,
            "zoom-in: (400,300) y should be >1, got {}",
            outside.y
        );
    }

    #[test]
    fn test_zoom_out() {
        let mut cam = Camera2D::new(800, 600);
        cam.zoom = 0.5;
        let proj = proj_matrix(&cam);

        // At zoom=0.5, visible half-extents are 800x600
        let edge = proj.project_point3(Vec3::new(800.0, 600.0, 0.0));
        assert_approx(edge.x, 1.0, "zoom-out edge x");
        assert_approx(edge.y, 1.0, "zoom-out edge y");
    }

    #[test]
    fn test_viewport_aspect_ratio() {
        let cam = Camera2D::new(1920, 1080);
        let proj = proj_matrix(&cam);

        // Half-width = 960, so (960, 0, 0) should be clip x=1
        let right = proj.project_point3(Vec3::new(960.0, 0.0, 0.0));
        assert_approx(right.x, 1.0, "aspect right edge x");

        // Half-height = 540, so (0, 540, 0) should be clip y=1
        let top = proj.project_point3(Vec3::new(0.0, 540.0, 0.0));
        assert_approx(top.y, 1.0, "aspect top edge y");
    }
}
