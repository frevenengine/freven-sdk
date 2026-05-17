use std::{
    fs,
    path::{Path, PathBuf},
};

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/visual_content_schema_v1")
}

fn read_fixture(relative: &str) -> String {
    let path = fixture_root().join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read fixture {}: {err}", path.display()))
}

fn collect_toml_files(root: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root)
        .unwrap_or_else(|err| panic!("failed to read fixture directory {}: {err}", root.display()))
    {
        let path = entry
            .unwrap_or_else(|err| panic!("failed to read fixture directory entry: {err}"))
            .path();

        if path.is_dir() {
            collect_toml_files(&path, out);
        } else if path
            .extension()
            .is_some_and(|ext| ext.to_string_lossy() == "toml")
        {
            out.push(path);
        }
    }
}

#[test]
fn required_visual_schema_fixtures_exist() {
    let required = [
        "README.md",
        "valid/cube_all/model_cube_all.toml",
        "valid/cube_all/material_stone.toml",
        "valid/cube_all/visual_stone.toml",
        "valid/cube_faces/model_cube_faces.toml",
        "valid/cube_faces/visual_grass_block.toml",
        "valid/cuboid_parts_framed_glass/model_framed_glass.toml",
        "valid/cuboid_parts_framed_glass/visual_framed_glass.toml",
        "valid/grass_tint_slots/model_grass_tint_cube.toml",
        "valid/grass_tint_slots/visual_tinted_grass_block.toml",
        "valid/emissive_material_no_light/material_glow_ore.toml",
        "valid/emissive_material_with_light/material_lamp.toml",
        "valid/families/rock_family.toml",
        "valid/families/soil_grass_family.toml",
        "valid/families/colored_glass_family.toml",
        "invalid/missing_model/visual_missing_model.toml",
        "invalid/unknown_material_slot/visual_unknown_material_slot.toml",
        "invalid/invalid_tint_source/visual_invalid_tint_source.toml",
        "invalid/invalid_family_combination/family_invalid_combination.toml",
        "invalid/renderer_internal_leak/visual_renderer_internal_leak.toml",
        "expected/consumer_map.toml",
        "expected/valid_keys.toml",
        "expected/invalid_diagnostics.toml",
    ];

    for relative in required {
        assert!(
            fixture_root().join(relative).is_file(),
            "missing visual content schema fixture: {relative}"
        );
    }
}

#[test]
fn valid_fixtures_do_not_author_renderer_or_runtime_internals() {
    let forbidden = [
        "renderer_slot",
        "material_id",
        "atlas_rect",
        "atlas_page",
        "texture_array_layer",
        "gpu_handle",
        "bevy_handle",
        "wgpu_handle",
        "runtime_block_id",
        "generated_cache_path",
    ];

    let mut files = Vec::new();
    collect_toml_files(&fixture_root().join("valid"), &mut files);

    for path in files {
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

        for term in forbidden {
            assert!(
                !text.contains(term),
                "valid fixture {} contains forbidden author-facing internal field `{term}`",
                path.display()
            );
        }
    }
}

#[test]
fn invalid_fixtures_are_mapped_to_expected_diagnostics() {
    let expected = read_fixture("expected/invalid_diagnostics.toml");

    for code in [
        "missing_model",
        "unknown_material_slot",
        "invalid_tint_source",
        "invalid_family_combination",
        "renderer_internal_leak",
    ] {
        assert!(
            expected.contains(code),
            "missing expected diagnostic code `{code}`"
        );
    }

    let mut invalid_files = Vec::new();
    collect_toml_files(&fixture_root().join("invalid"), &mut invalid_files);

    for path in invalid_files {
        let relative = path
            .strip_prefix(fixture_root())
            .expect("invalid fixture should be under fixture root")
            .to_string_lossy()
            .replace('\\', "/");

        assert!(
            expected.contains(&relative),
            "invalid fixture `{relative}` is not listed in expected diagnostics"
        );
    }
}

#[test]
fn fixture_readme_declares_consumer_boundary_and_phase_order() {
    let readme = read_fixture("README.md").to_lowercase();

    for required in [
        "freven-sdk",
        "freven-engine",
        "freven-devkit",
        "freven-boot",
        "freven-vanilla",
        "family expansion happens before runtime meshing",
    ] {
        assert!(
            readme.contains(required),
            "fixture README is missing required contract text `{required}`"
        );
    }
}

#[test]
fn authored_fixture_keys_are_namespaced() {
    let mut files = Vec::new();
    collect_toml_files(&fixture_root(), &mut files);

    for path in files {
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

        for line in text.lines() {
            let trimmed = line.trim();

            if !trimmed.starts_with("key = \"") {
                continue;
            }

            let value = trimmed.trim_start_matches("key = \"").trim_end_matches('"');

            assert!(
                value.contains(':'),
                "fixture key `{value}` in {} is not namespaced",
                path.display()
            );
            assert!(
                !value.contains(' '),
                "fixture key `{value}` in {} contains a space",
                path.display()
            );
        }
    }
}
