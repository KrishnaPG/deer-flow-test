//! Integration tests for scene and theme descriptors.

use deer_gui::scene::descriptor::{GeneratorParams, SceneDescriptor};
use deer_gui::scene::descriptor_config::DescriptorSceneConfig;
use deer_gui::scene::SceneConfig;
use deer_gui::theme::descriptor::ThemeDescriptor;

#[test]
fn t_desc_01_parse_ron_scene_descriptor() {
    let ron_str = r#"
        SceneDescriptor(
            name: "Test",
            ambient_audio: "audio/test.ogg",
            gltf_scene: None,
            theme: "TestTheme",
            generators: [
                (generator: "starfield", params: Starfield(
                    count: 100,
                    radius: 400.0,
                    emissive: (1.0, 1.0, 1.0, 1.0),
                )),
            ],
        )
    "#;
    let desc: SceneDescriptor = ron::from_str(ron_str).expect("should parse RON");
    assert_eq!(desc.name, "Test");
    assert_eq!(desc.ambient_audio, "audio/test.ogg");
    assert_eq!(desc.generators.len(), 1);
    assert_eq!(desc.generators[0].generator, "starfield");
    match &desc.generators[0].params {
        GeneratorParams::Starfield { count, radius, .. } => {
            assert_eq!(*count, 100);
            assert!((radius - 400.0).abs() < 0.01);
        }
        _ => panic!("expected Starfield params"),
    }
}

#[test]
fn t_desc_02_parse_ron_theme_descriptor() {
    let ron_str = r#"
        ThemeDescriptor(
            name: "Test",
            background: (0.1, 0.1, 0.1, 1.0),
            surface: (0.2, 0.2, 0.2, 1.0),
            accent: (0.0, 0.8, 1.0, 1.0),
            accent_secondary: (0.3, 0.5, 1.0, 1.0),
            text_primary: (0.9, 0.9, 0.9, 1.0),
            text_secondary: (0.5, 0.5, 0.5, 1.0),
            success: (0.2, 0.9, 0.4, 1.0),
            warning: (1.0, 0.75, 0.2, 1.0),
            error: (1.0, 0.3, 0.3, 1.0),
            panel_alpha: 0.75,
            panel_rounding: 8.0,
            star_emissive: (2.0, 2.0, 2.0, 1.0),
            monolith_emissive: (0.3, 0.5, 1.0, 1.0),
            trail_emissive: (0.0, 1.5, 0.8, 1.0),
            trail_base_color: (0.0, 0.8, 0.5, 1.0),
            monolith_glow_channels: (0.3, 0.5, 1.0),
            font_css_url: None,
        )
    "#;
    let desc: ThemeDescriptor = ron::from_str(ron_str).expect("should parse RON");
    assert_eq!(desc.name, "Test");

    let theme = desc.into_theme();
    assert_eq!(theme.name, "Test");
}

#[test]
fn t_desc_03_descriptor_to_scene_config() {
    let desc = SceneDescriptor {
        name: "TestScene".to_string(),
        ambient_audio: "audio/test.ogg".to_string(),
        gltf_scene: None,
        theme: "TestTheme".to_string(),
        generators: vec![],
    };
    let config = DescriptorSceneConfig::new(desc);
    assert_eq!(config.name(), "TestScene");
    assert_eq!(config.ambient_audio_track(), "audio/test.ogg");
}

#[test]
fn t_desc_04_loader_scan_finds_descriptors() {
    use deer_gui::scene::loader::SceneLoader;

    let dir = std::env::temp_dir().join("deer_gui_test_scenes");
    let _ = std::fs::create_dir_all(&dir);

    let ron_content = r#"
        SceneDescriptor(
            name: "LoaderTest",
            ambient_audio: "audio/test.ogg",
            gltf_scene: None,
            theme: "Test",
            generators: [],
        )
    "#;
    std::fs::write(dir.join("test.scene.ron"), ron_content).unwrap();

    let mut loader = SceneLoader::new(dir.clone());
    let count = loader.scan().expect("scan should succeed");
    assert!(count >= 1, "should find at least 1 descriptor");
    assert!(loader.get("LoaderTest").is_some());

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn t_desc_05_unknown_generator_logged() {
    // This test verifies that a descriptor with generators can be created
    // as a DescriptorSceneConfig without panicking. The unknown generators
    // are logged as warnings (tested via tracing subscriber in real app).
    let desc = SceneDescriptor {
        name: "UnknownGen".to_string(),
        ambient_audio: "audio/test.ogg".to_string(),
        gltf_scene: None,
        theme: "Test".to_string(),
        generators: vec![deer_gui::scene::descriptor::GeneratorSpec {
            generator: "nonexistent".to_string(),
            params: GeneratorParams::Starfield {
                count: 10,
                radius: 100.0,
                emissive: [1.0, 1.0, 1.0, 1.0],
            },
        }],
    };
    let config = DescriptorSceneConfig::new(desc);
    assert_eq!(config.name(), "UnknownGen");
}
