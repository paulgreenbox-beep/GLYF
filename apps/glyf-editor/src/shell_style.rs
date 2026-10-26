//! Makepad theme selection and semantic shell tokens for GLYF Editor.
//!
//! This module only resolves presentation values. It does not read or mutate
//! GLYF documents.

use designing_makepad::style::{
    PaletteStyle, ShapeStyle, StyleProfile, StyleTuning, SurfaceAppearance, SurfaceRole,
};
use makepad_widgets::{Cx, Vec4, script_eval};
use crate::ScriptMod;

/// Theme module path for a profile's palette and shape combination.
///
/// These names correspond to the twelve themes registered by Designing Makepad.
pub const fn makepad_theme_name(profile: StyleProfile) -> &'static str {
    match (profile.palette, profile.shape) {
        (PaletteStyle::Dark, ShapeStyle::Rounded) => "design_dark_rounded",
        (PaletteStyle::Dark, ShapeStyle::Soft) => "design_dark_soft",
        (PaletteStyle::Dark, ShapeStyle::Sharp) => "design_dark_sharp",
        (PaletteStyle::Dark, ShapeStyle::Angular) => "design_dark_angular",
        (PaletteStyle::Light, ShapeStyle::Rounded) => "design_light_rounded",
        (PaletteStyle::Light, ShapeStyle::Soft) => "design_light_soft",
        (PaletteStyle::Light, ShapeStyle::Sharp) => "design_light_sharp",
        (PaletteStyle::Light, ShapeStyle::Angular) => "design_light_angular",
        (PaletteStyle::Neon, ShapeStyle::Rounded) => "design_neon_rounded",
        (PaletteStyle::Neon, ShapeStyle::Soft) => "design_neon_soft",
        (PaletteStyle::Neon, ShapeStyle::Sharp) => "design_neon_sharp",
        (PaletteStyle::Neon, ShapeStyle::Angular) => "design_neon_angular",
    }
}

/// Apply the predeclared Designing Makepad theme for this profile.
///
/// The explicit arms are intentional: `script_eval!` resolves each registered
/// theme path against Makepad's script module.
pub fn apply_makepad_palette(cx: &mut Cx, profile: StyleProfile) {
    match (profile.palette, profile.shape) {
        (PaletteStyle::Dark, ShapeStyle::Rounded) => {
            script_eval!(cx, {mod.theme = mod.themes.design_dark_rounded});
        }
        (PaletteStyle::Dark, ShapeStyle::Soft) => {
            script_eval!(cx, {mod.theme = mod.themes.design_dark_soft});
        }
        (PaletteStyle::Dark, ShapeStyle::Sharp) => {
            script_eval!(cx, {mod.theme = mod.themes.design_dark_sharp});
        }
        (PaletteStyle::Dark, ShapeStyle::Angular) => {
            script_eval!(cx, {mod.theme = mod.themes.design_dark_angular});
        }
        (PaletteStyle::Light, ShapeStyle::Rounded) => {
            script_eval!(cx, {mod.theme = mod.themes.design_light_rounded});
        }
        (PaletteStyle::Light, ShapeStyle::Soft) => {
            script_eval!(cx, {mod.theme = mod.themes.design_light_soft});
        }
        (PaletteStyle::Light, ShapeStyle::Sharp) => {
            script_eval!(cx, {mod.theme = mod.themes.design_light_sharp});
        }
        (PaletteStyle::Light, ShapeStyle::Angular) => {
            script_eval!(cx, {mod.theme = mod.themes.design_light_angular});
        }
        (PaletteStyle::Neon, ShapeStyle::Rounded) => {
            script_eval!(cx, {mod.theme = mod.themes.design_neon_rounded});
        }
        (PaletteStyle::Neon, ShapeStyle::Soft) => {
            script_eval!(cx, {mod.theme = mod.themes.design_neon_soft});
        }
        (PaletteStyle::Neon, ShapeStyle::Sharp) => {
            script_eval!(cx, {mod.theme = mod.themes.design_neon_sharp});
        }
        (PaletteStyle::Neon, ShapeStyle::Angular) => {
            script_eval!(cx, {mod.theme = mod.themes.design_neon_angular});
        }
    }
}

/// The resolved colors and sizing roles most commonly needed by the app shell.
#[derive(Clone, Copy, Debug)]
pub struct ShellStyleTokens {
    pub window: SurfaceAppearance,
    pub toolbar: SurfaceAppearance,
    pub canvas: SurfaceAppearance,
    pub menu: SurfaceAppearance,
    pub status: SurfaceAppearance,
    pub text_primary: Vec4,
    pub text_secondary: Vec4,
    pub accent: Vec4,
    pub separator: Vec4,
    pub toolbar_height: f64,
    pub menu_item_height: f64,
}

/// Resolve shared style tokens without touching application or document state.
pub fn shell_style_tokens(profile: StyleProfile, tuning: StyleTuning) -> ShellStyleTokens {
    let tokens = profile.tokens(tuning);
    // Popup content must fully obscure the toolbar/canvas underneath. The
    // design-system surface treatment intentionally tints even Elevated
    // surfaces, but that makes menu text compete with controls behind it.
    let mut menu = tokens.surface(SurfaceRole::Popup);
    menu.fill.w = 1.0;
    ShellStyleTokens {
        window: tokens.surface(SurfaceRole::Window),
        toolbar: tokens.surface(SurfaceRole::Toolbar),
        canvas: tokens.surface(SurfaceRole::Canvas),
        menu,
        status: tokens.surface(SurfaceRole::Toolbar),
        text_primary: tokens.text_primary,
        text_secondary: tokens.text_secondary,
        accent: tokens.accent,
        separator: tokens.separator,
        toolbar_height: tokens.toolbar_height,
        menu_item_height: tokens.menu_item_height,
    }
}

#[cfg(test)]
mod tests {
    use super::{makepad_theme_name, shell_style_tokens};
    use designing_makepad::style::{PaletteStyle, ShapeStyle, StyleProfile, StyleTuning, SurfaceStyle};

    #[test]
    fn every_palette_shape_pair_resolves_to_its_registered_theme() {
        let palettes = [PaletteStyle::Dark, PaletteStyle::Light, PaletteStyle::Neon];
        let shapes = [
            ShapeStyle::Rounded,
            ShapeStyle::Soft,
            ShapeStyle::Sharp,
            ShapeStyle::Angular,
        ];

        for palette in palettes {
            for shape in shapes {
                let profile = StyleProfile {
                    palette,
                    shape,
                    ..StyleProfile::production()
                };
                let expected = format!("design_{}_{}", palette.code(), shape.code());
                assert_eq!(makepad_theme_name(profile), expected);
            }
        }
    }

    #[test]
    fn popup_surface_is_opaque_even_for_glass_profile() {
        for palette in [PaletteStyle::Dark, PaletteStyle::Light, PaletteStyle::Neon] {
            for surface in [SurfaceStyle::Flat, SurfaceStyle::Elevated, SurfaceStyle::Glass, SurfaceStyle::Neumorphic] {
                let profile = StyleProfile { palette, surface, ..StyleProfile::production() };
                assert_eq!(shell_style_tokens(profile, StyleTuning::default()).menu.fill.w, 1.0);
            }
        }
    }
}
