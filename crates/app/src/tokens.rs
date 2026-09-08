//! Application-owned tokens that are intentionally outside GPUI Kit's theme schema.
//!
//! The JSON theme owns the kit's semantic colors, fonts, and radii. These values
//! describe xsaber-specific density and file-type presentation so components do
//! not scatter literals through the app shell.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color(pub &'static str);

impl Color {
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

pub const BACKGROUND: Color = Color("#0c0d10");
pub const PRIMARY: Color = Color("#ff6b3d");
pub const PRIMARY_HOVER: Color = Color("#ff8b5f");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtBadgePalette {
    pub archive: Color,
    pub document: Color,
    pub image: Color,
    pub script: Color,
    pub config: Color,
    pub binary: Color,
    pub foreground: Color,
}

pub const EXT_BADGE: ExtBadgePalette = ExtBadgePalette {
    archive: Color("#b39bff"),
    document: Color("#7aa2ff"),
    image: Color("#3ecfb2"),
    script: Color("#ffb020"),
    config: Color("#ff8b5f"),
    binary: Color("#4b525e"),
    foreground: Color("#0a0b0e"),
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RowHeights {
    pub file: f32,
    pub column_header: f32,
    pub site: f32,
    pub control: f32,
    pub tab: f32,
    pub drawer_tab: f32,
    pub strip: f32,
    pub pane_header: f32,
    pub title_bar: f32,
    pub toolbar: f32,
}

pub const ROW_HEIGHTS: RowHeights = RowHeights {
    file: 25.0,
    column_header: 26.0,
    site: 27.0,
    control: 28.0,
    tab: 30.0,
    drawer_tab: 32.0,
    strip: 34.0,
    pane_header: 36.0,
    title_bar: 40.0,
    toolbar: 46.0,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabelStyle {
    pub font_size: f32,
    pub line_height: f32,
    pub letter_spacing_em: f32,
}

pub const SMALL_LABEL: LabelStyle = LabelStyle {
    font_size: 9.5,
    line_height: 1.0,
    letter_spacing_em: 0.08,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_tokens_match_the_handoff_scale() {
        assert_eq!(ROW_HEIGHTS.file, 25.0);
        assert_eq!(ROW_HEIGHTS.toolbar, 46.0);
        assert_eq!(SMALL_LABEL.font_size, 9.5);
        assert_eq!(SMALL_LABEL.letter_spacing_em, 0.08);
    }

    #[test]
    fn extension_badges_use_named_palette_colors() {
        assert_eq!(EXT_BADGE.archive.as_str(), "#b39bff");
        assert_eq!(EXT_BADGE.foreground.as_str(), "#0a0b0e");
    }
}
