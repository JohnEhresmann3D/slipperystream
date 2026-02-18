use image::RgbaImage;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct AtlasMetadata {
    version: String,
    atlas_id: String,
    texture: AtlasTexture,
    sprites: Vec<AtlasSprite>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AtlasTexture {
    path: String,
    width: u32,
    height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct AtlasSprite {
    sprite_id: String,
    name: String,
    source_path: String,
    rect_px: AtlasRectPx,
    uv: AtlasUvRect,
    pivot: AtlasPivot,
}

#[derive(Debug, Serialize, Deserialize)]
struct AtlasRectPx {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct AtlasUvRect {
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct AtlasPivot {
    x: f32,
    y: f32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct IdRegistryFile {
    entries: Vec<IdRegistryEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct IdRegistryEntry {
    sprite_id: String,
    source_hash: String,
    last_known_path: String,
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
    let mut id_registry = load_id_registry(&id_registry_path_for(&atlas_json_output))?;
    if id_registry.entries.is_empty() {
        seed_registry_from_existing_metadata(&atlas_json_output, &mut id_registry)?;
    }
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
        let source_hash = hash_rgba8_bytes(image.as_raw());
        let sprite_name = source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("sprite")
            .to_string();
        let sprite_id = resolve_or_assign_sprite_id(&mut id_registry, &rel_source, &source_hash);

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
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create atlas output dir '{}': {e}",
                parent.display()
            )
        })?;
    }
    if let Some(parent) = atlas_json_output.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create atlas metadata output dir '{}': {e}",
                parent.display()
            )
        })?;
    }

    let png_tmp = temporary_output_path(&atlas_png_output);
    atlas
        .save_with_format(&png_tmp, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to write '{}': {e}", png_tmp.display()))?;

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
    let json_tmp = temporary_output_path(&atlas_json_output);
    fs::write(&json_tmp, json)
        .map_err(|e| format!("Failed to write '{}': {e}", json_tmp.display()))?;
    let id_registry_path = id_registry_path_for(&atlas_json_output);
    let id_registry_json = serde_json::to_string_pretty(&id_registry).map_err(|e| {
        format!(
            "Failed to serialize id registry '{}': {e}",
            id_registry_path.display()
        )
    })?;
    let id_registry_tmp = temporary_output_path(&id_registry_path);
    fs::write(&id_registry_tmp, id_registry_json)
        .map_err(|e| format!("Failed to write '{}': {e}", id_registry_tmp.display()))?;

    promote_outputs_transactional(&[
        (&png_tmp, &atlas_png_output),
        (&json_tmp, &atlas_json_output),
        (&id_registry_tmp, &id_registry_path),
    ])?;

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

fn hash_rgba8_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    format!("{digest:x}")
}

fn id_registry_path_for(atlas_json_output: &Path) -> PathBuf {
    atlas_json_output.with_extension("ids.json")
}

fn load_id_registry(path: &Path) -> Result<IdRegistryFile, String> {
    if !path.exists() {
        return Ok(IdRegistryFile::default());
    }

    let raw = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read id registry '{}': {e}", path.display()))?;
    serde_json::from_str::<IdRegistryFile>(&raw)
        .map_err(|e| format!("Failed to parse id registry '{}': {e}", path.display()))
}

fn seed_registry_from_existing_metadata(
    atlas_json_output: &Path,
    id_registry: &mut IdRegistryFile,
) -> Result<(), String> {
    if !atlas_json_output.exists() {
        return Ok(());
    }

    let raw = fs::read_to_string(atlas_json_output).map_err(|e| {
        format!(
            "Failed to read existing atlas metadata '{}': {e}",
            atlas_json_output.display()
        )
    })?;
    let metadata = serde_json::from_str::<AtlasMetadata>(&raw).map_err(|e| {
        format!(
            "Failed to parse existing atlas metadata '{}': {e}",
            atlas_json_output.display()
        )
    })?;
    for sprite in metadata.sprites {
        if id_registry
            .entries
            .iter()
            .any(|entry| entry.sprite_id == sprite.sprite_id)
        {
            continue;
        }
        id_registry.entries.push(IdRegistryEntry {
            sprite_id: sprite.sprite_id,
            source_hash: String::new(),
            last_known_path: sprite.source_path,
        });
    }
    Ok(())
}

fn resolve_or_assign_sprite_id(
    id_registry: &mut IdRegistryFile,
    source_path: &str,
    source_hash: &str,
) -> String {
    if let Some(entry) = id_registry
        .entries
        .iter_mut()
        .find(|entry| entry.last_known_path == source_path)
    {
        entry.source_hash = source_hash.to_string();
        return entry.sprite_id.clone();
    }

    let mut hash_matches = id_registry
        .entries
        .iter_mut()
        .filter(|entry| entry.source_hash == source_hash);
    if let Some(entry) = hash_matches.next() {
        // Reuse IDs across rename/move when content stays identical.
        entry.last_known_path = source_path.to_string();
        return entry.sprite_id.clone();
    }

    let sprite_id = Uuid::new_v4().to_string();
    id_registry.entries.push(IdRegistryEntry {
        sprite_id: sprite_id.clone(),
        source_hash: source_hash.to_string(),
        last_known_path: source_path.to_string(),
    });
    sprite_id
}

fn temporary_output_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("output");
    path.with_file_name(format!("{file_name}.tmp"))
}

fn promote_temporary_file(temp_path: &Path, final_path: &Path) -> Result<(), String> {
    if final_path.exists() {
        fs::remove_file(final_path).map_err(|e| {
            format!(
                "Failed to replace existing output '{}': {e}",
                final_path.display()
            )
        })?;
    }
    fs::rename(temp_path, final_path).map_err(|e| {
        format!(
            "Failed to move temporary output '{}' -> '{}': {e}",
            temp_path.display(),
            final_path.display()
        )
    })
}

fn promote_outputs_transactional(pairs: &[(&Path, &Path)]) -> Result<(), String> {
    let mut backups: HashMap<PathBuf, PathBuf> = HashMap::new();
    let mut promoted: Vec<PathBuf> = Vec::new();

    for (_, final_path) in pairs {
        if final_path.exists() {
            let backup_path = final_path.with_extension("bak.tmp");
            fs::rename(final_path, &backup_path).map_err(|e| {
                format!(
                    "Failed to stage backup '{}' -> '{}': {e}",
                    final_path.display(),
                    backup_path.display()
                )
            })?;
            backups.insert((*final_path).to_path_buf(), backup_path);
        }
    }

    for (temp_path, final_path) in pairs {
        match promote_temporary_file(temp_path, final_path) {
            Ok(()) => promoted.push((*final_path).to_path_buf()),
            Err(err) => {
                for promoted_path in promoted.iter().rev() {
                    let _ = fs::remove_file(promoted_path);
                    if let Some(backup_path) = backups.get(promoted_path) {
                        let _ = fs::rename(backup_path, promoted_path);
                    }
                }
                for (final_path, backup_path) in backups {
                    if !final_path.exists() {
                        let _ = fs::rename(backup_path, final_path);
                    }
                }
                return Err(err);
            }
        }
    }

    for (_, backup_path) in backups {
        let _ = fs::remove_file(backup_path);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn test_temp_path(hint: &str) -> PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("sme_packer_test_{}_{}.tmp", hint, nanos))
    }

    // ---- Path/string utilities ----

    #[test]
    fn test_normalize_path_replaces_backslashes() {
        let result = normalize_path_for_json(Path::new("foo\\bar\\baz.png"));
        assert_eq!(result, "foo/bar/baz.png");
    }

    #[test]
    fn test_normalize_path_preserves_forward_slashes() {
        let result = normalize_path_for_json(Path::new("foo/bar/baz.png"));
        assert_eq!(result, "foo/bar/baz.png");
    }

    #[test]
    fn test_hash_rgba8_bytes_deterministic() {
        let bytes_a = vec![0u8, 1, 2, 3, 4, 5, 6, 7];
        let bytes_b = vec![10u8, 20, 30, 40, 50, 60, 70, 80];

        let hash_a1 = hash_rgba8_bytes(&bytes_a);
        let hash_a2 = hash_rgba8_bytes(&bytes_a);
        let hash_b = hash_rgba8_bytes(&bytes_b);

        // Same input produces same hash
        assert_eq!(hash_a1, hash_a2);

        // Different input produces different hash
        assert_ne!(hash_a1, hash_b);

        // Valid hex string: lowercase, 64 chars for SHA-256
        assert_eq!(hash_a1.len(), 64);
        assert!(hash_a1.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(hash_a1, hash_a1.to_lowercase());
    }

    #[test]
    fn test_id_registry_path_for() {
        let result = id_registry_path_for(Path::new("atlas.json"));
        assert_eq!(result, PathBuf::from("atlas.ids.json"));
    }

    #[test]
    fn test_temporary_output_path() {
        let result = temporary_output_path(Path::new("output/atlas.png"));
        assert_eq!(result, PathBuf::from("output/atlas.png.tmp"));
    }

    // ---- ID Registry ----

    #[test]
    fn test_load_id_registry_missing_file() {
        let path = test_temp_path("nonexistent_registry");
        // Ensure it doesn't exist
        let _ = fs::remove_file(&path);

        let result = load_id_registry(&path);
        assert!(result.is_ok());
        let registry = result.unwrap();
        assert!(registry.entries.is_empty());
    }

    #[test]
    fn test_load_id_registry_valid_file() {
        let path = test_temp_path("valid_registry");
        let registry = IdRegistryFile {
            entries: vec![
                IdRegistryEntry {
                    sprite_id: "id-aaa".to_string(),
                    source_hash: "hash-aaa".to_string(),
                    last_known_path: "sprites/hero.png".to_string(),
                },
                IdRegistryEntry {
                    sprite_id: "id-bbb".to_string(),
                    source_hash: "hash-bbb".to_string(),
                    last_known_path: "sprites/enemy.png".to_string(),
                },
            ],
        };
        let json = serde_json::to_string_pretty(&registry).expect("serialize");
        fs::write(&path, json).expect("write");

        let loaded = load_id_registry(&path).expect("load");
        assert_eq!(loaded.entries.len(), 2);
        assert_eq!(loaded.entries[0].sprite_id, "id-aaa");
        assert_eq!(loaded.entries[0].source_hash, "hash-aaa");
        assert_eq!(loaded.entries[0].last_known_path, "sprites/hero.png");
        assert_eq!(loaded.entries[1].sprite_id, "id-bbb");
        assert_eq!(loaded.entries[1].source_hash, "hash-bbb");
        assert_eq!(loaded.entries[1].last_known_path, "sprites/enemy.png");

        // Cleanup
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_resolve_sprite_id_by_path() {
        let mut registry = IdRegistryFile {
            entries: vec![IdRegistryEntry {
                sprite_id: "existing-id-123".to_string(),
                source_hash: "old_hash".to_string(),
                last_known_path: "a.png".to_string(),
            }],
        };

        let result = resolve_or_assign_sprite_id(&mut registry, "a.png", "some_hash");
        assert_eq!(result, "existing-id-123");
    }

    #[test]
    fn test_resolve_sprite_id_by_hash() {
        let mut registry = IdRegistryFile {
            entries: vec![IdRegistryEntry {
                sprite_id: "hash-matched-id".to_string(),
                source_hash: "abc123".to_string(),
                last_known_path: "old.png".to_string(),
            }],
        };

        let result = resolve_or_assign_sprite_id(&mut registry, "renamed.png", "abc123");
        assert_eq!(result, "hash-matched-id");

        // Verify last_known_path was updated
        assert_eq!(registry.entries[0].last_known_path, "renamed.png");
    }

    #[test]
    fn test_resolve_sprite_id_new_assignment() {
        let mut registry = IdRegistryFile { entries: vec![] };

        let result = resolve_or_assign_sprite_id(&mut registry, "brand_new.png", "new_hash_xyz");

        // Should return a non-empty string (UUID)
        assert!(!result.is_empty());
        // UUID format: 8-4-4-4-12 hex chars with dashes = 36 chars
        assert_eq!(result.len(), 36);
        assert!(result.contains('-'));

        // Should have added an entry
        assert_eq!(registry.entries.len(), 1);
        assert_eq!(registry.entries[0].sprite_id, result);
        assert_eq!(registry.entries[0].source_hash, "new_hash_xyz");
        assert_eq!(registry.entries[0].last_known_path, "brand_new.png");
    }

    #[test]
    fn test_resolve_sprite_id_updates_hash_on_path_match() {
        let mut registry = IdRegistryFile {
            entries: vec![IdRegistryEntry {
                sprite_id: "stable-id-456".to_string(),
                source_hash: "old_hash".to_string(),
                last_known_path: "a.png".to_string(),
            }],
        };

        let result = resolve_or_assign_sprite_id(&mut registry, "a.png", "new_hash");

        // Should return the same sprite_id
        assert_eq!(result, "stable-id-456");

        // source_hash should be updated
        assert_eq!(registry.entries[0].source_hash, "new_hash");
    }

    // ---- File operations ----

    #[test]
    fn test_promote_temporary_file_success() {
        let temp = test_temp_path("promote_temp");
        let final_path = test_temp_path("promote_final");

        // Ensure final doesn't exist
        let _ = fs::remove_file(&final_path);

        fs::write(&temp, "temporary content").expect("write temp");

        let result = promote_temporary_file(&temp, &final_path);
        assert!(result.is_ok());

        // Final should have the content
        let content = fs::read_to_string(&final_path).expect("read final");
        assert_eq!(content, "temporary content");

        // Temp should be gone
        assert!(!temp.exists());

        // Cleanup
        let _ = fs::remove_file(&final_path);
    }

    #[test]
    fn test_promote_temporary_file_overwrites_existing() {
        let temp = test_temp_path("overwrite_temp");
        let final_path = test_temp_path("overwrite_final");

        fs::write(&final_path, "old content").expect("write final");
        fs::write(&temp, "new content").expect("write temp");

        let result = promote_temporary_file(&temp, &final_path);
        assert!(result.is_ok());

        let content = fs::read_to_string(&final_path).expect("read final");
        assert_eq!(content, "new content");

        assert!(!temp.exists());

        // Cleanup
        let _ = fs::remove_file(&final_path);
    }

    #[test]
    fn test_promote_outputs_transactional_success() {
        let temp1 = test_temp_path("txn_temp1");
        let final1 = test_temp_path("txn_final1");
        let temp2 = test_temp_path("txn_temp2");
        let final2 = test_temp_path("txn_final2");

        // Ensure finals don't exist
        let _ = fs::remove_file(&final1);
        let _ = fs::remove_file(&final2);

        fs::write(&temp1, "content one").expect("write temp1");
        fs::write(&temp2, "content two").expect("write temp2");

        let pairs: Vec<(&Path, &Path)> = vec![(&temp1, &final1), (&temp2, &final2)];
        let result = promote_outputs_transactional(&pairs);
        assert!(result.is_ok());

        let content1 = fs::read_to_string(&final1).expect("read final1");
        let content2 = fs::read_to_string(&final2).expect("read final2");
        assert_eq!(content1, "content one");
        assert_eq!(content2, "content two");

        assert!(!temp1.exists());
        assert!(!temp2.exists());

        // Cleanup
        let _ = fs::remove_file(&final1);
        let _ = fs::remove_file(&final2);
    }
}
