#![forbid(unsafe_code)]

pub mod tokens;

#[cfg(test)]
mod theme_tests {
    use gpui_kit::component::ThemeSet;
    use serde_json::Value;

    const THEME_JSON: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/themes/xsaber-dark.json"
    ));

    #[test]
    fn theme_asset_is_a_theme_set_with_dotted_kit_fields() {
        let value: Value = serde_json::from_str(THEME_JSON).expect("valid theme JSON");
        let themes = value
            .get("themes")
            .and_then(Value::as_array)
            .expect("ThemeSet themes array");
        assert_eq!(themes.len(), 1);
        let theme = &themes[0];
        assert_eq!(theme["mode"], "dark");
        assert_eq!(theme["font.family"], "Barlow");
        assert_eq!(theme["mono_font.family"], "IBM Plex Mono");
        assert_eq!(theme["radius"], 5);
        assert_eq!(theme["radius.lg"], 10);
        assert_eq!(
            theme["colors"]["background"],
            crate::tokens::BACKGROUND.as_str()
        );
        assert_eq!(
            theme["colors"]["primary.background"],
            crate::tokens::PRIMARY.as_str()
        );
        assert_eq!(
            theme["colors"]["primary.hover.background"],
            crate::tokens::PRIMARY_HOVER.as_str()
        );
    }

    #[test]
    fn theme_asset_round_trips_through_gpui_kit_theme_set() {
        let set: ThemeSet = serde_json::from_str(THEME_JSON).expect("GPUI Kit ThemeSet");
        let encoded = serde_json::to_string(&set).expect("serialize GPUI Kit ThemeSet");
        let round_tripped: ThemeSet =
            serde_json::from_str(&encoded).expect("round-trip GPUI Kit ThemeSet");
        assert_eq!(round_tripped.themes.len(), 1);
        assert_eq!(round_tripped.themes[0].name.as_ref(), "xsaber dark");
        assert_eq!(
            round_tripped.themes[0].mode,
            gpui_kit::component::ThemeMode::Dark
        );
    }
}
