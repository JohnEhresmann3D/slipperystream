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
