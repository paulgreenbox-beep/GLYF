//! Small naming, path and presentation helpers shared by editor adapters.

use super::*;

pub(crate) fn tab_title(path: &str, index: usize) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed.starts_with("Untitled") || trimmed == "project.glyf" {
        format!("Untitled {}", index + 1)
    } else {
        Path::new(trimmed)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(trimmed)
            .to_owned()
    }
}
pub(crate) fn supported_image(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .as_deref(),
        Some("png" | "jpg" | "jpeg" | "webp" | "bmp" | "ico")
    )
}

pub(crate) fn unique_part_name(parts: &[GlyfPart], base: &str) -> String {
    if !parts.iter().any(|part| part.name == base) {
        return base.to_owned();
    }
    for index in 2..10_000 {
        let candidate = format!("{base}_{index}");
        if !parts.iter().any(|part| part.name == candidate) {
            return candidate;
        }
    }
    format!("part_{}", parts.len() + 1)
}

pub(crate) fn annotation_geometry_summary(annotation: &GlyfAnnotation) -> String {
    match &annotation.geometry {
        GlyfAnnotationGeometry::Point { .. } => "Point".into(),
        GlyfAnnotationGeometry::Region { .. } => "Region".into(),
    }
}

pub(crate) fn localized_annotation_geometry_summary(annotation: &GlyfAnnotation, language: Language) -> String {
    if language == Language::English {
        return annotation_geometry_summary(annotation);
    }
    match &annotation.geometry {
        GlyfAnnotationGeometry::Point { .. } => "Точка".into(),
        GlyfAnnotationGeometry::Region { .. } => "Область".into(),
    }
}

pub(crate) fn localized_style_profile_summary(profile: StyleProfile, language: Language) -> String {
    if language == Language::English {
        return profile.summary();
    }
    let palette = match profile.palette {
        PaletteStyle::Dark => "Тёмная",
        PaletteStyle::Light => "Светлая",
        PaletteStyle::Neon => "Неоновая",
    };
    let shape = match profile.shape {
        ShapeStyle::Rounded => "Скруглённая",
        ShapeStyle::Soft => "Мягкая",
        ShapeStyle::Sharp => "Строгая",
        ShapeStyle::Angular => "Угловатая",
    };
    let surface = match profile.surface {
        SurfaceStyle::Flat => "Плоская",
        SurfaceStyle::Elevated => "Объёмная",
        SurfaceStyle::Glass => "Стеклянная",
        SurfaceStyle::Neumorphic => "Неоморфная",
    };
    let motion = match profile.motion {
        MotionStyle::None => "Без анимации",
        MotionStyle::Reduced => "Уменьшенная",
        MotionStyle::Subtle => "Лёгкая",
        MotionStyle::Smooth => "Плавная",
    };
    format!("{palette} · {shape} · {surface} · {motion}")
}

pub(crate) fn annotation_coordinates_summary(annotation: &GlyfAnnotation) -> String {
    match &annotation.geometry {
        GlyfAnnotationGeometry::Point { rel } => format!("rel: {:.4} {:.4}", rel.x, rel.y),
        GlyfAnnotationGeometry::Region { rel1, rel2 } => format!(
            "rel1: {:.4} {:.4}; rel2: {:.4} {:.4}",
            rel1.x, rel1.y, rel2.x, rel2.y
        ),
    }
}
pub(crate) fn unique_annotation_name(annotations: &[GlyfAnnotation], base: &str) -> String {
    if !annotations.iter().any(|annotation| annotation.name == base) {
        return base.to_owned();
    }
    for index in 2..10_000 {
        let candidate = format!("{base}_{index}");
        if !annotations
            .iter()
            .any(|annotation| annotation.name == candidate)
        {
            return candidate;
        }
    }
    format!("annotation_{}", annotations.len() + 1)
}

pub(crate) fn unique_name(images: &[EditorImage], base: &str) -> String {
    if !images.iter().any(|image| image.name == base) {
        return base.to_owned();
    }
    for index in 2..10_000 {
        let candidate = format!("{base}_{index}");
        if !images.iter().any(|image| image.name == candidate) {
            return candidate;
        }
    }
    format!("image_{}", images.len() + 1)
}

pub(crate) fn with_extension(path: &str, extension: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.extension().is_some_and(|value| value == extension) {
        path
    } else {
        path.with_extension(extension)
    }
}
