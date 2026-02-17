use image::RgbaImage;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct AtlasMetadata {
    version: String,
    atlas_id: String,
    texture: AtlasTexture,
    sprites: Vec<AtlasSprite>,
}

#[derive(Debug, Serialize)]
struct AtlasTexture {
    path: String,
    width: u32,
    height: u32,
}

#[derive(Debug, Serialize)]
struct AtlasSprite {
    sprite_id: String,
    name: String,
    source_path: String,
    rect_px: AtlasRectPx,
    uv: AtlasUvRect,
    pivot: AtlasPivot,
}

#[derive(Debug, Serialize)]
struct AtlasRectPx {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[derive(Debug, Serialize)]
struct AtlasUvRect {
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
}

#[derive(Debug, Serialize)]
struct AtlasPivot {
    x: f32,
    y: f32,
}

fn usage() -> String {
    "Usage: cargo run -p sme_atlas_packer -- <input_dir> <atlas_png_output> <atlas_json_output> [atlas_size]\nExample: cargo run -p sme_atlas_packer -- assets/textures assets/generated/m4_sample_atlas.png assets/generated/m4_sample_atlas.json 512".to_string()
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 || args.len() > 5 {
        return Err(usage());
    }

    let input_dir = PathBuf::from(&args[1]);
    let atlas_png_output = PathBuf::from(&args[2]);
    let atlas_json_output = PathBuf::from(&args[3]);
    let atlas_size = if args.len() == 5 {
        args[4]
            .parse::<u32>()
            .map_err(|e| format!("Invalid atlas_size '{}': {e}", args[4]))?
    } else {
        512
    };
    if atlas_size == 0 {
        return Err("atlas_size must be > 0".to_string());
    }

    let mut input_files: Vec<PathBuf> = fs::read_dir(&input_dir)
        .map_err(|e| format!("Failed to read input dir '{}': {e}", input_dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("png"))
        .collect();
    input_files.sort();

    if input_files.is_empty() {
        return Err(format!(
            "No .png files found in input directory '{}'",
            input_dir.display()
        ));
    }

    let mut atlas = RgbaImage::new(atlas_size, atlas_size);
    let mut sprites = Vec::new();
    let mut x = 0u32;
    let mut y = 0u32;
    let mut row_height = 0u32;
    let padding = 1u32;

    for source_path in input_files {
        let image = image::open(&source_path)
            .map_err(|e| format!("Failed to open '{}': {e}", source_path.display()))?
            .to_rgba8();
        let (w, h) = image.dimensions();

        if w + padding * 2 > atlas_size || h + padding * 2 > atlas_size {
            return Err(format!(
                "Sprite '{}' ({}x{}) does not fit in atlas {}x{}",
                source_path.display(),
                w,
                h,
                atlas_size,
                atlas_size
            ));
        }

        if x + w + padding > atlas_size {
            x = 0;
            y += row_height;
            row_height = 0;
        }
        if y + h + padding > atlas_size {
            return Err(format!(
                "Atlas overflow while packing '{}'. Increase atlas_size.",
                source_path.display()
            ));
        }

        image::imageops::replace(&mut atlas, &image, x as i64, y as i64);

        let rel_source = normalize_path_for_json(&source_path);
        let sprite_name = source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("sprite")
            .to_string();
        let sprite_id = Uuid::new_v5(&Uuid::NAMESPACE_URL, rel_source.as_bytes()).to_string();

        sprites.push(AtlasSprite {
            sprite_id,
            name: sprite_name,
            source_path: rel_source,
            rect_px: AtlasRectPx { x, y, w, h },
            uv: AtlasUvRect {
                u0: x as f32 / atlas_size as f32,
                v0: y as f32 / atlas_size as f32,
                u1: (x + w) as f32 / atlas_size as f32,
                v1: (y + h) as f32 / atlas_size as f32,
            },
            pivot: AtlasPivot { x: 0.5, y: 0.5 },
        });

        x += w + padding;
        row_height = row_height.max(h + padding);
    }

    if let Some(parent) = atlas_png_output.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create atlas output dir '{}': {e}", parent.display()))?;
    }
    if let Some(parent) = atlas_json_output.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create atlas metadata output dir '{}': {e}",
                parent.display()
            )
        })?;
    }

    atlas
        .save(&atlas_png_output)
        .map_err(|e| format!("Failed to write '{}': {e}", atlas_png_output.display()))?;

    let atlas_id = atlas_json_output
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("atlas")
        .to_string();
    let metadata = AtlasMetadata {
        version: "0.1".to_string(),
        atlas_id,
        texture: AtlasTexture {
            path: normalize_path_for_json(&atlas_png_output),
            width: atlas_size,
            height: atlas_size,
        },
        sprites,
    };
    let json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Failed to serialize atlas metadata: {e}"))?;
    fs::write(&atlas_json_output, json)
        .map_err(|e| format!("Failed to write '{}': {e}", atlas_json_output.display()))?;

    println!(
        "Packed {} sprites -> {} and {}",
        metadata.sprites.len(),
        atlas_png_output.display(),
        atlas_json_output.display()
    );
    Ok(())
}

fn normalize_path_for_json(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
