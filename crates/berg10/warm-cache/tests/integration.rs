use berg10_warm_cache::WarmCacheConfig;

#[test]
fn warm_cache_config_paths() {
    let config = WarmCacheConfig::with_base_dir("/tmp/berg10-test");
    assert_eq!(
        config.checkout_path("music-by-year"),
        "/tmp/berg10-test/checkouts/music-by-year"
    );
    assert_eq!(
        config.content_path("abc123"),
        "/tmp/berg10-test/content/ab/c1/23.blob"
    );
}

#[test]
fn warm_cache_default_config() {
    let config = WarmCacheConfig::default();
    assert_eq!(config.base_dir, ".berg10");
    assert_eq!(config.checkouts_dir, "checkouts");
    assert_eq!(config.content_dir, "content");
}
