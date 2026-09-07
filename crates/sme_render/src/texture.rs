use wgpu;

const MAX_IMAGE_BYTES: u64 = 64 * 1024 * 1024;

fn decode_image(bytes: &[u8], max_dimension: u32) -> Result<image::RgbaImage, String> {
    if bytes.len() as u64 > MAX_IMAGE_BYTES {
        return Err("encoded image exceeds 64 MiB budget".into());
    }
    let reader = || {
        image::ImageReader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|e| e.to_string())
    };
    let (width, height) = reader()?.into_dimensions().map_err(|e| e.to_string())?;
    validate_dimensions(width, height, max_dimension)?;
    let mut reader = reader()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(max_dimension);
    limits.max_image_height = Some(max_dimension);
    limits.max_alloc = Some(MAX_IMAGE_BYTES);
    reader.limits(limits);
    Ok(reader.decode().map_err(|e| e.to_string())?.to_rgba8())
}

fn validate_dimensions(width: u32, height: u32, max_dimension: u32) -> Result<(), String> {
    if width == 0 || height == 0 || width > max_dimension || height > max_dimension {
        return Err(format!(
            "invalid image dimensions {width}x{height}; device limit {max_dimension}"
        ));
    }
    let size = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixels| pixels.checked_mul(4));
    if size.is_none_or(|size| size > MAX_IMAGE_BYTES) {
        return Err("decoded RGBA image exceeds 64 MiB budget".into());
    }
    Ok(())
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: (u32, u32),
}

impl Texture {
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Self {
        Self::try_from_bytes(device, queue, bytes, label).expect("Failed to load texture")
    }

    /// Validates/decode-bounds external assets before GPU upload. Device loss/OOM
    /// still belongs to the GPU host lifecycle, not this image validation result.
    pub fn try_from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self, String> {
        let img = decode_image(bytes, device.limits().max_texture_dimension_2d)
            .map_err(|e| format!("Texture '{label}': {e}"))?;
        Ok(Self::from_rgba8(
            device,
            queue,
            &img,
            img.width(),
            img.height(),
            label,
        ))
    }

    pub fn from_rgba8(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rgba: &[u8],
        width: u32,
        height: u32,
        label: &str,
    ) -> Self {
        let expected_len = (width as usize)
            .checked_mul(height as usize)
            .and_then(|wh| wh.checked_mul(4))
            .expect("texture dimensions overflow usize");
        assert_eq!(
            rgba.len(),
            expected_len,
            "from_rgba8 expects width*height*4 bytes"
        );

        let tex_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: tex_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            tex_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            size: (width, height),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn corrupt_and_oversized_images_fail_before_gpu_upload() {
        assert!(decode_image(b"not a PNG", 4096).is_err());
        assert!(validate_dimensions(0, 1, 4096).is_err());
        assert!(validate_dimensions(4097, 1, 4096).is_err());
        assert!(validate_dimensions(8192, 8192, 8192).is_err());
        assert!(validate_dimensions(u32::MAX, u32::MAX, u32::MAX).is_err());
        let png = include_bytes!("../../../assets/textures/test_sprite.png");
        assert!(decode_image(png, 4096).is_ok());
        assert!(decode_image(png, 1).is_err());
    }
}
