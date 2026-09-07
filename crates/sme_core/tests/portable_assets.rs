//! The same byte APIs are available to filesystem, embedded and fetch-based hosts.
use sme_core::{
    animation::parse_animation,
    animation_registry::AnimationRegistry,
    atlas::{parse_atlas, MultiAtlasRegistry},
    collision::parse_collision,
    scene::parse_scene,
};

#[test]
fn embedded_scene_atlas_collision_and_animation_resolve_without_filesystem() {
    let scene = parse_scene(include_bytes!("../../../assets/scenes/m4_scene.json")).unwrap();
    let atlas = parse_atlas(include_bytes!(
        "../../../assets/generated/m4_sample_atlas.json"
    ))
    .unwrap();
    let mut registry = MultiAtlasRegistry::new();
    registry.add_atlas("sample", atlas).unwrap();
    let mut id = None;
    for sprite in scene.layers.iter().flat_map(|layer| &layer.sprites) {
        if let Some(sprite_id) = &sprite.sprite_id {
            assert!(registry.resolve(sprite_id).is_some());
            id = Some(sprite_id);
        }
    }
    let id = id.expect("fixture must exercise atlas references");
    let data = serde_json::json!({ "version":"0.1", "animation_id":"sample", "animations": {
        "idle": { "looping":true, "frames":[{"sprite_id":id,"duration_ms":100}] }
    }});
    let mut animations = AnimationRegistry::new();
    animations
        .add_file(parse_animation(&serde_json::to_vec(&data).unwrap()).unwrap())
        .unwrap();
    animations.validate_sprites(&registry).unwrap();
    assert!(animations.resolve_clip(Some("sample"), "idle").is_some());
    let collision = parse_collision(include_bytes!(
        "../../../assets/collision/m3_collision.json"
    ))
    .unwrap();
    assert!(collision.cell_size > 0);
}

#[test]
fn malformed_generated_data_returns_errors_instead_of_panicking() {
    assert!(parse_scene(b"{}").is_err());
    assert!(parse_atlas(b"{}").is_err());
    assert!(parse_collision(
        br#"{"version":"0.1","collision_id":"bad","cell_size":0,"width":1,"height":1,"solids":[]}"#
    )
    .is_err());
    let overflow = br#"{"version":"0.1","animation_id":"bad","animations":{"idle":{"frames":[{"sprite_id":"x","duration_ms":18446744073709551615}]}}}"#;
    assert!(parse_animation(overflow).is_err());
}
