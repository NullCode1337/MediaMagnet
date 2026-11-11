use mm_core::settings::{force_desktop_fields, sanitize_settings, Settings, DESKTOP_ONLY_KEYS};

#[test]
fn sanitize_drops_unknown_and_defaults_missing() {
    let raw = serde_json::json!({ "dark_mode": false, "bogus": 1, "accent_hue": 400.4 });
    let (value, report) = sanitize_settings(&raw);
    assert!(report.dropped.contains(&"bogus".to_string()));
    assert!(report.coerced.contains(&"accent_hue".to_string()));
    assert_eq!(value["accent_hue"], 360);
    assert!(report.defaulted.contains(&"yt_format".to_string()));
}

#[test]
fn desktop_fields_are_stripped_from_server_view() {
    let settings = Settings {
        download_path: "/tmp/x".into(),
        ..Settings::default()
    };
    let view = settings.server_view();
    for key in DESKTOP_ONLY_KEYS {
        assert!(view.get(key).is_none(), "{key} leaked");
    }
    assert!(view.get("yt_format").is_some());
}

#[test]
fn force_desktop_fields_restores_local_values() {
    let current = Settings::default();
    let mut incoming = Settings {
        always_on_top: true,
        download_path: "/etc".into(),
        user_agent: "x".into(),
        ..Settings::default()
    };
    force_desktop_fields(&mut incoming, &current);
    assert!(!incoming.always_on_top);
    assert_eq!(incoming.download_path, "");
    assert_eq!(incoming.user_agent, "x");
}

#[test]
fn overrides_merge_over_base_and_drop_invalid_values() {
    let base = Settings {
        yt_format: "worst".into(),
        ..Settings::default()
    };
    let overrides = serde_json::json!({
        "yt_format": "best",
        "yt_embed_subs": true,
        "accent_hue": "nope",
        "bogus": 1,
    });
    let merged = Settings::with_overrides(&base, &overrides).unwrap();
    assert_eq!(merged.yt_format, "best");
    assert!(merged.yt_embed_subs);
    assert_eq!(merged.accent_hue, base.accent_hue);
    assert!(Settings::with_overrides(&base, &serde_json::json!([])).is_err());
}

#[test]
fn legacy_format_default_is_migrated_but_a_custom_choice_is_kept() {
    let legacy = serde_json::json!({
        "settings_version": 1,
        "yt_format": "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best",
    });
    let settings = Settings::from_value(&legacy).unwrap();
    assert_eq!(settings.yt_format, Settings::default().yt_format);

    let custom = serde_json::json!({
        "settings_version": 1,
        "yt_format": "worst",
    });
    let settings = Settings::from_value(&custom).unwrap();
    assert_eq!(settings.yt_format, "worst");
}

#[test]
fn settings_round_trip_through_file() {
    let dir = std::env::temp_dir().join(format!("mm-settings-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let settings = Settings {
        spotdl_format: "flac".into(),
        ..Settings::default()
    };
    settings.save_to(&dir).unwrap();
    assert_eq!(Settings::load_from(&dir), settings);
    std::fs::remove_dir_all(&dir).unwrap();
}
