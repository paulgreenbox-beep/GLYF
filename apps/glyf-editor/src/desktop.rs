use std::{
    env, fs,
    path::{Path, PathBuf},
};

use designing_makepad::{
    MotionStyle, PaletteStyle, ShapeStyle, StylePreset, StyleProfile, SurfaceStyle,
};

pub const RECENT_MENU_CAPACITY: usize = 10;
const SETTINGS_VERSION: u32 = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontSizeChoice {
    Small,
    #[default]
    Normal,
    Large,
}

impl FontSizeChoice {
    pub const ALL: [Self; 3] = [Self::Small, Self::Normal, Self::Large];
    pub const fn code(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Normal => "normal",
            Self::Large => "large",
        }
    }
    pub const fn label(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Normal => "Normal",
            Self::Large => "Large",
        }
    }
    pub fn from_code(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "small" => Some(Self::Small),
            "normal" => Some(Self::Normal),
            "large" => Some(Self::Large),
            _ => None,
        }
    }
    const fn scale(self) -> f32 {
        match self {
            Self::Small => 0.9,
            Self::Normal => 1.0,
            Self::Large => 1.15,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiScaleChoice {
    #[default]
    Percent100,
    Percent110,
    Percent125,
    Percent150,
}

impl UiScaleChoice {
    pub const ALL: [Self; 4] = [
        Self::Percent100,
        Self::Percent110,
        Self::Percent125,
        Self::Percent150,
    ];
    pub const fn percent(self) -> u16 {
        match self {
            Self::Percent100 => 100,
            Self::Percent110 => 110,
            Self::Percent125 => 125,
            Self::Percent150 => 150,
        }
    }
    pub const fn code(self) -> &'static str {
        match self {
            Self::Percent100 => "100",
            Self::Percent110 => "110",
            Self::Percent125 => "125",
            Self::Percent150 => "150",
        }
    }
    pub const fn label(self) -> &'static str {
        match self {
            Self::Percent100 => "100%",
            Self::Percent110 => "110%",
            Self::Percent125 => "125%",
            Self::Percent150 => "150%",
        }
    }
    pub fn from_percent(value: &str) -> Option<Self> {
        match value.trim() {
            "100" => Some(Self::Percent100),
            "110" => Some(Self::Percent110),
            "125" => Some(Self::Percent125),
            "150" => Some(Self::Percent150),
            _ => None,
        }
    }
    const fn factor(self) -> f32 {
        self.percent() as f32 / 100.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DensityChoice {
    Compact,
    #[default]
    Comfortable,
}

impl DensityChoice {
    pub const ALL: [Self; 2] = [Self::Compact, Self::Comfortable];
    pub const fn code(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Comfortable => "comfortable",
        }
    }
    pub const fn label(self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Comfortable => "Comfortable",
        }
    }
    pub fn from_code(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "compact" => Some(Self::Compact),
            "comfortable" => Some(Self::Comfortable),
            _ => None,
        }
    }
    const fn factor(self) -> f32 {
        match self {
            Self::Compact => 0.8,
            Self::Comfortable => 1.0,
        }
    }
}

/// Centralized font-size tokens, expressed in logical pixels before platform scaling.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypographyTokens {
    pub interface: f32,
    pub heading: f32,
    pub caption: f32,
}

/// Centralized spacing tokens, expressed in logical pixels before platform scaling.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpacingTokens {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub panel_padding: f32,
    pub row_gap: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecentMenuEntry {
    pub path: String,
    pub label: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    English,
    Russian,
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Russian => "ru",
        }
    }

    pub fn from_code(value: &str) -> Self {
        if value.eq_ignore_ascii_case("ru") || value.eq_ignore_ascii_case("русский") {
            Self::Russian
        } else {
            Self::English
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemeChoice {
    System,
    Light,
    #[default]
    Dark,
}

impl ThemeChoice {
    pub fn code(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn from_code(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Self::System,
        }
    }

    /// Compatibility projection for startup paths that still need the legacy
    /// two-state Makepad theme. `StyleProfile` remains the persisted source of
    /// truth, including Neon and custom palettes.
    pub fn from_style_profile(profile: StyleProfile) -> Self {
        if profile.palette == PaletteStyle::Light {
            Self::Light
        } else {
            Self::Dark
        }
    }

    fn into_legacy_style_profile(self) -> StyleProfile {
        let mut profile = StyleProfile::production();
        if self == Self::Light {
            profile.palette = PaletteStyle::Light;
        }
        profile
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesktopSettings {
    pub language: Language,
    pub theme: ThemeChoice,
    pub default_zoom: f64,
    pub restore_session: bool,
    pub recent_file_count: usize,
    pub recent_files: Vec<String>,
    pub style_profile: StyleProfile,
    pub font_size: FontSizeChoice,
    pub ui_scale: UiScaleChoice,
    pub density: DensityChoice,
    pub panel_resources: bool,
    pub panel_layers: bool,
    pub panel_inspector: bool,
    pub panel_status_bar: bool,
    pub confirm_destructive: bool,
}

impl Default for DesktopSettings {
    fn default() -> Self {
        Self {
            language: Language::English,
            theme: ThemeChoice::Dark,
            default_zoom: 1.0,
            restore_session: false,
            recent_file_count: 10,
            recent_files: Vec::new(),
            style_profile: StyleProfile::production(),
            font_size: FontSizeChoice::Normal,
            ui_scale: UiScaleChoice::Percent100,
            density: DensityChoice::Comfortable,
            panel_resources: false,
            panel_layers: false,
            panel_inspector: false,
            panel_status_bar: true,
            confirm_destructive: true,
        }
    }
}

impl DesktopSettings {
    pub fn typography_tokens(&self) -> TypographyTokens {
        let scale = self.font_size.scale() * self.ui_scale.factor();
        TypographyTokens {
            interface: 14.0 * scale,
            heading: 18.0 * scale,
            caption: 12.0 * scale,
        }
    }

    pub fn spacing_tokens(&self) -> SpacingTokens {
        let scale = self.density.factor() * self.ui_scale.factor();
        SpacingTokens {
            xs: 4.0 * scale,
            sm: 8.0 * scale,
            md: 12.0 * scale,
            lg: 16.0 * scale,
            panel_padding: 12.0 * scale,
            row_gap: 8.0 * scale,
        }
    }

    pub fn load() -> Self {
        settings_path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|contents| parse_settings(&contents).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let Some(path) = settings_path() else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(path, serialize_settings(self)).map_err(|error| error.to_string())
    }

    pub fn add_recent(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref().display().to_string();
        self.recent_files.retain(|item| item != &path);
        self.recent_files.insert(0, path);
        self.recent_files.truncate(self.recent_file_count.max(1));
    }

    pub fn remove_recent(&mut self, path: &str) {
        self.recent_files.retain(|item| item != path);
    }

    pub fn clear_recent(&mut self) {
        self.recent_files.clear();
    }

    /// Removes paths which cannot be reopened before presenting them as a
    /// command. The removed value lets the caller persist the cleanup.
    pub fn prune_missing_recent(&mut self) -> bool {
        let before = self.recent_files.len();
        self.recent_files.retain(|path| Path::new(path).is_file());
        self.recent_files.len() != before
    }
}

/// Produces compact labels while retaining the persisted MRU order and full
/// path payload. Duplicate filenames gain a parent-folder hint; if that still
/// collides, the complete path remains visible and unambiguous.
pub fn recent_menu_entries(paths: &[String]) -> Vec<RecentMenuEntry> {
    let file_names = paths
        .iter()
        .map(|path| {
            Path::new(path)
                .file_name()
                .and_then(|name| name.to_str())
                .filter(|name| !name.is_empty())
                .unwrap_or(path)
                .to_owned()
        })
        .collect::<Vec<_>>();
    let mut labels = file_names.clone();
    for (index, name) in file_names.iter().enumerate() {
        if file_names
            .iter()
            .filter(|candidate| *candidate == name)
            .count()
            > 1
        {
            let parent = Path::new(&paths[index])
                .parent()
                .and_then(|parent| parent.file_name())
                .and_then(|parent| parent.to_str())
                .filter(|parent| !parent.is_empty())
                .unwrap_or("…");
            labels[index] = format!("{name} — {parent}");
        }
    }
    for index in 0..labels.len() {
        if labels
            .iter()
            .filter(|candidate| *candidate == &labels[index])
            .count()
            > 1
        {
            labels[index] = paths[index].clone();
        }
    }
    paths
        .iter()
        .cloned()
        .zip(labels)
        .map(|(path, label)| RecentMenuEntry { path, label })
        .collect()
}

pub fn settings_path() -> Option<PathBuf> {
    if let Ok(root) = env::var("APPDATA") {
        return Some(
            PathBuf::from(root)
                .join("Glyf Editor")
                .join("settings.conf"),
        );
    }
    if let Ok(root) = env::var("XDG_CONFIG_HOME") {
        return Some(
            PathBuf::from(root)
                .join("glyf-editor")
                .join("settings.conf"),
        );
    }
    env::var("HOME")
        .ok()
        .map(|root| PathBuf::from(root).join(".config/glyf-editor/settings.conf"))
}

fn serialize_settings(settings: &DesktopSettings) -> String {
    let recent = settings
        .recent_files
        .iter()
        .map(|path| {
            path.replace('\\', "\\\\")
                .replace('\n', "\\n")
                .replace('|', "\\|")
        })
        .collect::<Vec<_>>()
        .join("|");
    let compatibility_theme = ThemeChoice::from_style_profile(settings.style_profile);
    format!(
        "version={}\nlanguage={}\ntheme={}\ndefault_zoom={:.6}\nrestore_session={}\nrecent_file_count={}\nrecent_files={}\nstyle_preset={}\nstyle_palette={}\nstyle_shape={}\nstyle_surface={}\nstyle_motion={}\ninterface_font_size={}\nui_scale={}\ndensity={}\nshow_resources={}\nshow_layers={}\nshow_inspector={}\nshow_status={}\nconfirm_destructive={}\n",
        SETTINGS_VERSION,
        settings.language.code(),
        compatibility_theme.code(),
        settings.default_zoom,
        settings.restore_session,
        settings.recent_file_count,
        recent,
        settings
            .style_profile
            .preset()
            .map(StylePreset::code)
            .unwrap_or("custom"),
        settings.style_profile.palette.code(),
        settings.style_profile.shape.code(),
        settings.style_profile.surface.code(),
        settings.style_profile.motion.code(),
        settings.font_size.code(),
        settings.ui_scale.code(),
        settings.density.code(),
        settings.panel_resources,
        settings.panel_layers,
        settings.panel_inspector,
        settings.panel_status_bar,
        settings.confirm_destructive,
    )
}

fn parse_settings(contents: &str) -> Result<DesktopSettings, String> {
    let mut settings = DesktopSettings::default();
    let mut recognized = false;
    let mut corrupt = false;
    let mut legacy_theme = None;
    let mut has_style_profile = false;
    let mut style_preset = None;
    let mut style_palette = None;
    let mut style_shape = None;
    let mut style_surface = None;
    let mut style_motion = None;
    for line in contents.lines() {
        let line = line.trim();
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        match key {
            "language"
            | "theme"
            | "default_zoom"
            | "restore_session"
            | "recent_file_count"
            | "recent_files"
            | "style_preset"
            | "style_palette"
            | "style_shape"
            | "style_surface"
            | "style_motion"
            | "interface_font_size"
            | "font_size"
            | "ui_scale"
            | "density"
            | "show_resources"
            | "show_layers"
            | "show_inspector"
            | "show_status"
            | "confirm_destructive" => recognized = true,
            _ => continue,
        }
        match key {
            "language" => settings.language = Language::from_code(value),
            "theme" => legacy_theme = Some(ThemeChoice::from_code(value)),
            "default_zoom" => match value.parse::<f64>() {
                Ok(zoom) if zoom.is_finite() => settings.default_zoom = zoom.clamp(0.25, 4.0),
                _ => corrupt = true,
            },
            "restore_session" => match parse_bool(value) {
                Some(v) => settings.restore_session = v,
                None => corrupt = true,
            },
            "recent_file_count" => match value.parse::<usize>() {
                Ok(n) => settings.recent_file_count = n.clamp(1, RECENT_MENU_CAPACITY),
                Err(_) => corrupt = true,
            },
            "recent_files" => {
                settings.recent_files = decode_recent_files(value);
            }
            "style_preset" => {
                has_style_profile = true;
                match parse_style_preset(value) {
                    Some(preset) => style_preset = Some(preset),
                    None if value.eq_ignore_ascii_case("custom") => style_preset = None,
                    None => corrupt = true,
                }
            }
            "style_palette" => {
                style_palette = Some(PaletteStyle::from_code(value));
                has_style_profile = true;
            }
            "style_shape" => {
                style_shape = Some(ShapeStyle::from_code(value));
                has_style_profile = true;
            }
            "style_surface" => {
                style_surface = Some(SurfaceStyle::from_code(value));
                has_style_profile = true;
            }
            "style_motion" => {
                style_motion = Some(MotionStyle::from_code(value));
                has_style_profile = true;
            }
            "interface_font_size" | "font_size" => match FontSizeChoice::from_code(value) {
                Some(v) => settings.font_size = v,
                None => corrupt = true,
            },
            "ui_scale" => match UiScaleChoice::from_percent(value) {
                Some(v) => settings.ui_scale = v,
                None => corrupt = true,
            },
            "density" => match DensityChoice::from_code(value) {
                Some(v) => settings.density = v,
                None => corrupt = true,
            },
            "show_resources" => match parse_bool(value) {
                Some(v) => settings.panel_resources = v,
                None => corrupt = true,
            },
            "show_layers" => match parse_bool(value) {
                Some(v) => settings.panel_layers = v,
                None => corrupt = true,
            },
            "show_inspector" => match parse_bool(value) {
                Some(v) => settings.panel_inspector = v,
                None => corrupt = true,
            },
            "show_status" => match parse_bool(value) {
                Some(v) => settings.panel_status_bar = v,
                None => corrupt = true,
            },
            "confirm_destructive" => match parse_bool(value) {
                Some(v) => settings.confirm_destructive = v,
                None => corrupt = true,
            },
            _ => {}
        }
    }
    if !recognized {
        return Err("settings contain no recognized keys".into());
    }
    if corrupt {
        return Err("settings contain malformed values".into());
    }
    if has_style_profile {
        let mut profile = style_preset
            .map(StyleProfile::from_preset)
            .or_else(|| legacy_theme.map(ThemeChoice::into_legacy_style_profile))
            .unwrap_or_else(StyleProfile::production);
        if let Some(palette) = style_palette {
            profile.palette = palette;
        }
        if let Some(shape) = style_shape {
            profile.shape = shape;
        }
        if let Some(surface) = style_surface {
            profile.surface = surface;
        }
        if let Some(motion) = style_motion {
            profile.motion = motion;
        }
        settings.style_profile = profile;
    } else if let Some(theme) = legacy_theme {
        // Older settings files had only `theme`; migrate it once into the
        // richer profile. System followed the production dark theme at startup.
        settings.style_profile = theme.into_legacy_style_profile();
    }
    settings.theme = ThemeChoice::from_style_profile(settings.style_profile);
    settings.recent_files.truncate(settings.recent_file_count);
    Ok(settings)
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_style_preset(value: &str) -> Option<StylePreset> {
    StylePreset::ALL
        .into_iter()
        .find(|preset| preset.code().eq_ignore_ascii_case(value.trim()))
}

fn decode_recent_files(value: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut current = String::new();
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '|' => {
                if !current.is_empty() {
                    paths.push(std::mem::take(&mut current));
                }
            }
            '\\' => match chars.peek().copied() {
                Some('|') => {
                    chars.next();
                    current.push('|');
                }
                Some('n') => {
                    chars.next();
                    current.push('\n');
                }
                Some('\\') => {
                    chars.next();
                    current.push('\\');
                }
                _ => current.push('\\'),
            },
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        paths.push(current);
    }
    paths
}

pub fn text(key: &str, language: Language) -> &'static str {
    match (key, language) {
        ("settings.general", Language::Russian) => "Основные",
        ("settings.general", _) => "General",
        ("settings.appearance", Language::Russian) => "Внешний вид",
        ("settings.appearance", _) => "Appearance",
        ("settings.editor", Language::Russian) => "Редактор",
        ("settings.editor", _) => "Editor",
        ("settings.files", Language::Russian) => "Файлы",
        ("settings.files", _) => "Files",
        ("settings.accessibility", Language::Russian) => "Доступность",
        ("settings.accessibility", _) => "Accessibility",
        ("settings.language", Language::Russian) => "Язык интерфейса",
        ("settings.language", _) => "Interface language",
        ("settings.restore_session", Language::Russian) => "Восстанавливать предыдущую сессию",
        ("settings.restore_session", _) => "Restore previous session",
        ("settings.recent_count", Language::Russian) => "Количество недавних файлов",
        ("settings.recent_count", _) => "Recent files count",
        ("settings.preset", Language::Russian) => "Профиль стиля",
        ("settings.preset", _) => "Style profile",
        ("settings.palette", Language::Russian) => "Цветовая палитра",
        ("settings.palette", _) => "Palette",
        ("settings.shape", Language::Russian) => "Форма",
        ("settings.shape", _) => "Shape",
        ("settings.surface", Language::Russian) => "Поверхность",
        ("settings.surface", _) => "Surface",
        ("settings.motion", Language::Russian) => "Анимация",
        ("settings.motion", _) => "Motion",
        ("settings.font_size", Language::Russian) => "Размер шрифта",
        ("settings.font_size", _) => "Font size",
        ("settings.ui_scale", Language::Russian) => "Масштаб интерфейса",
        ("settings.ui_scale", _) => "UI scale",
        ("settings.density", Language::Russian) => "Плотность интерфейса",
        ("settings.density", _) => "Interface density",
        ("settings.default_zoom", Language::Russian) => "Масштаб холста по умолчанию",
        ("settings.default_zoom", _) => "Default canvas zoom",
        ("settings.resources", Language::Russian) => "Панель ресурсов",
        ("settings.resources", _) => "Resources panel",
        ("settings.layers", Language::Russian) => "Панель слоёв",
        ("settings.layers", _) => "Layers panel",
        ("settings.inspector", Language::Russian) => "Инспектор",
        ("settings.inspector", _) => "Inspector",
        ("settings.status_bar", Language::Russian) => "Строка состояния",
        ("settings.status_bar", _) => "Status bar",
        ("settings.confirm_destructive", Language::Russian) => {
            "Подтверждать удаление и другие опасные действия"
        }
        ("settings.confirm_destructive", _) => "Confirm destructive actions",
        ("settings.reset_appearance", Language::Russian) => "Сбросить оформление",
        ("settings.reset_appearance", _) => "Reset appearance",
        ("settings.reset_all", Language::Russian) => "Сбросить все настройки",
        ("settings.reset_all", _) => "Reset all settings",
        ("settings.close", Language::Russian) => "Закрыть настройки",
        ("settings.close", _) => "Close settings",
        ("settings.accent", Language::Russian) => "Акцентный цвет",
        ("settings.accent", _) => "Accent",
        ("settings.reduced_motion", Language::Russian) => "Уменьшить анимацию",
        ("settings.reduced_motion", _) => "Reduced motion",
        ("settings.high_contrast", Language::Russian) => "Высокая контрастность",
        ("settings.high_contrast", _) => "High contrast",
        ("file", Language::Russian) => "Файл",
        ("edit", Language::Russian) => "Правка",
        ("view", Language::Russian) => "Вид",
        ("window", Language::Russian) => "Окно",
        ("language", Language::Russian) => "Язык",
        ("help", Language::Russian) => "Справка",
        ("file", _) => "File",
        ("edit", _) => "Edit",
        ("view", _) => "View",
        ("window", _) => "Window",
        ("language", _) => "Language",
        ("help", _) => "Help",
        ("new", Language::Russian) => "Создать",
        ("new", _) => "New",
        ("open", Language::Russian) => "Открыть...",
        ("open", _) => "Open...",
        ("open_recent" | "file.open_recent", Language::Russian) => "Недавние файлы",
        ("open_recent" | "file.open_recent", _) => "Open Recent",
        ("no_recent_files" | "file.no_recent", Language::Russian) => "Нет недавних файлов",
        ("no_recent_files" | "file.no_recent", _) => "No recent files",
        ("clear_recent" | "file.clear_recent", Language::Russian) => "Очистить список",
        ("clear_recent" | "file.clear_recent", _) => "Clear Recent",
        ("save", Language::Russian) => "Сохранить",
        ("save", _) => "Save",
        ("save_as", Language::Russian) => "Сохранить как...",
        ("save_as", _) => "Save As...",
        ("save_pack", Language::Russian) => "Сохранить и упаковать",
        ("save_pack", _) => "Save & Pack",
        ("close", Language::Russian) => "Закрыть",
        ("close", _) => "Close",
        ("exit", Language::Russian) => "Выход",
        ("exit", _) => "Exit",
        ("undo", Language::Russian) => "Отменить",
        ("undo", _) => "Undo",
        ("redo", Language::Russian) => "Повторить",
        ("redo", _) => "Redo",
        ("cut", Language::Russian) => "Вырезать",
        ("cut", _) => "Cut",
        ("copy", Language::Russian) => "Копировать",
        ("copy", _) => "Copy",
        ("paste", Language::Russian) => "Вставить",
        ("paste", _) => "Paste",
        ("duplicate", Language::Russian) => "Дублировать",
        ("duplicate", _) => "Duplicate",
        ("delete", Language::Russian) => "Удалить",
        ("delete", _) => "Delete",
        ("select_all", Language::Russian) => "Выделить всё",
        ("select_all", _) => "Select All",
        ("zoom_in", Language::Russian) => "Увеличить",
        ("zoom_in", _) => "Zoom In",
        ("zoom_out", Language::Russian) => "Уменьшить",
        ("zoom_out", _) => "Zoom Out",
        ("fit", Language::Russian) => "Вписать",
        ("fit", _) => "Fit",
        ("resources", Language::Russian) => "Ресурсы",
        ("resources", _) => "Resources",
        ("layers", Language::Russian) => "Слои",
        ("layers", _) => "Layers",
        ("inspector", Language::Russian) => "Инспектор",
        ("inspector", _) => "Inspector",
        ("settings", Language::Russian) => "Настройки",
        ("settings", _) => "Settings",
        ("about", Language::Russian) => "О программе",
        ("about", _) => "About Glyf Editor",
        ("shortcuts", Language::Russian) => "Горячие клавиши",
        ("shortcuts", _) => "Keyboard Shortcuts",
        ("english", _) => "English",
        ("russian", _) => "Русский",
        ("system", Language::Russian) => "Системная",
        ("system", _) => "System",
        ("light", Language::Russian) => "Светлая",
        ("light", _) => "Light",
        ("dark", Language::Russian) => "Тёмная",
        ("dark", _) => "Dark",
        ("editor.images_heading", Language::Russian) => "ИЗОБРАЖЕНИЯ",
        ("editor.images_heading", _) => "IMAGES",
        ("editor.no_images_yet", Language::Russian) => "Пока нет изображений",
        ("editor.no_images_yet", _) => "No images yet",
        ("editor.no_images", Language::Russian) => "Нет изображений",
        ("editor.no_images", _) => "No images",
        ("editor.image_path_placeholder", Language::Russian) => {
            "Путь к изображению"
        }
        ("editor.image_path_placeholder", _) => "PNG / JPG / WEBP / BMP / ICO path",
        ("editor.add_image", Language::Russian) => "+ Добавить",
        ("editor.add_image", _) => "+ Add",
        ("editor.remove_image", Language::Russian) => "Удалить",
        ("editor.remove_image", _) => "Remove",
        ("editor.metadata_heading", Language::Russian) => "Метаданные ресурса",
        ("editor.metadata_heading", _) => "Resource metadata",
        ("editor.image_name_placeholder", Language::Russian) => "имя изображения",
        ("editor.image_name_placeholder", _) => "image name",
        ("editor.source_label", Language::Russian) => "Источник:",
        ("editor.source_label", _) => "Source:",
        ("editor.resource_comment_placeholder", Language::Russian) => "комментарий к ресурсу",
        ("editor.resource_comment_placeholder", _) => "resource comment",
        ("editor.update_resource", Language::Russian) => "Обновить ресурс",
        ("editor.update_resource", _) => "Update resource",
        ("editor.open_source", Language::Russian) => "Открыть источник",
        ("editor.open_source", _) => "Open Source",
        ("editor.layers_heading", Language::Russian) => "СЛОИ · ПОРЯДОК",
        ("editor.layers_heading", _) => "LAYERS · Z-ORDER",
        ("editor.no_parts_yet", Language::Russian) => "Пока нет частей",
        ("editor.no_parts_yet", _) => "No parts yet",
        ("editor.no_parts", Language::Russian) => "Нет частей",
        ("editor.no_parts", _) => "No parts",
        ("editor.add_part", Language::Russian) => "+ Часть",
        ("editor.add_part", _) => "+ Part",
        ("editor.add_clip", Language::Russian) => "+ Фрагмент",
        ("editor.add_clip", _) => "+ Clip",
        ("editor.delete_part", Language::Russian) => "Удалить",
        ("editor.delete_part", _) => "Delete",
        ("editor.edit_source_crop", Language::Russian) => "Изменить фрагмент",
        ("editor.edit_source_crop", _) => "Edit source crop",
        ("editor.up", Language::Russian) => "Вверх",
        ("editor.up", _) => "Up",
        ("editor.down", Language::Russian) => "Вниз",
        ("editor.down", _) => "Down",
        ("editor.back", Language::Russian) => "Позади всех",
        ("editor.back", _) => "Back",
        ("editor.front", Language::Russian) => "Поверх всех",
        ("editor.front", _) => "Front",
        ("editor.layers_hint", Language::Russian) => {
            "Перетащите часть на холст, чтобы разместить её. Дважды щёлкните, чтобы изменить имя и комментарий."
        }
        ("editor.layers_hint", _) => {
            "Drag a part on Canvas to place it. Double-click it to edit name and comment."
        }
        ("editor.annotations_heading", Language::Russian) => "АННОТАЦИИ",
        ("editor.annotations_heading", _) => "ANNOTATIONS",
        ("editor.no_annotations_yet", Language::Russian) => "Пока нет аннотаций",
        ("editor.no_annotations_yet", _) => "No annotations yet",
        ("editor.no_annotations", Language::Russian) => "Нет аннотаций",
        ("editor.no_annotations", _) => "No annotations",
        ("editor.edit_annotation", Language::Russian) => "Изменить",
        ("editor.edit_annotation", _) => "Edit",
        ("editor.delete_annotation", Language::Russian) => "Удалить",
        ("editor.delete_annotation", _) => "Delete",
        ("editor.annotations_hint", Language::Russian) => {
            "Комментарии к точкам и областям добавляются на холсте"
        }
        ("editor.annotations_hint", _) => "Point + Region comments live on Canvas",
        ("editor.no_selection", Language::Russian) => "Ничего не выбрано",
        ("editor.no_selection", _) => "No selection",
        ("editor.role_label", Language::Russian) => "Роль:",
        ("editor.role_label", _) => "Role:",
        ("editor.coordinates_label", Language::Russian) => "Координаты:",
        ("editor.coordinates_label", _) => "Coordinates:",
        ("editor.comment_label", Language::Russian) => "Комментарий:",
        ("editor.comment_label", _) => "Comment:",
        ("editor.add_annotation", Language::Russian) => "Добавить аннотацию",
        ("editor.add_annotation", _) => "Add annotation",
        ("editor.annotation_prompt", Language::Russian) => "Что здесь находится?",
        ("editor.annotation_prompt", _) => "What is here?",
        ("editor.cancel", Language::Russian) => "Отмена",
        ("editor.cancel", _) => "Cancel",
        ("editor.done", Language::Russian) => "Готово",
        ("editor.done", _) => "Done",
        ("editor.preview_empty", Language::Russian) => {
            "Пока нет частей\n\nДобавьте изображение, затем создайте часть\nиз всего изображения или его фрагмента."
        }
        ("editor.preview_empty", _) => {
            "No parts yet\n\nAdd an image, then create a part\nfrom the whole image or a clipped region."
        }
        ("editor.composition", Language::Russian) => "КОМПОЗИЦИЯ",
        ("editor.composition", _) => "COMPOSITION",
        ("editor.source_mode", Language::Russian) => "РЕЖИМ ИСТОЧНИКА",
        ("editor.source_mode", _) => "SOURCE MODE",
        ("editor.saved", Language::Russian) => "СОХРАНЕНО",
        ("editor.saved", _) => "SAVED",
        ("editor.unsaved", Language::Russian) => "НЕ СОХРАНЕНО",
        ("editor.unsaved", _) => "UNSAVED",
        ("editor.ready", Language::Russian) => "Готово",
        ("editor.ready", _) => "Ready",
        _ => "",
    }
}

/// Localizes known editor status messages while preserving paths, errors, and
/// other dynamic values. Unknown messages are returned unchanged.
pub fn localized_status(message: &str, language: Language) -> String {
    if language == Language::English {
        return message.to_owned();
    }

    let exact = match message {
        "Ready — create a project or add an image" => {
            "Готово — создайте проект или добавьте изображение"
        }
        "Active document" => "Активный документ",
        "Removed unavailable recent files" => "Недоступные недавние файлы удалены",
        "No recent files" => "Нет недавних файлов",
        "Recent files cleared" => "Список недавних файлов очищен",
        "Recent file no longer exists" => "Недавний файл больше не существует",
        "Settings saved" => "Настройки сохранены",
        "Overlay dismissed" => "Всплывающая панель закрыта",
        "Open cancelled" => "Открытие отменено",
        "Save As cancelled" => "Сохранение как отменено",
        "Add image cancelled" => "Добавление изображения отменено",
        "Interaction cancelled" => "Действие отменено",
        "Nothing to undo" => "Нечего отменять",
        "Nothing to redo" => "Нечего повторять",
        "Select an annotation or part first" => "Сначала выберите аннотацию или часть",
        "Clipboard is empty" => "Буфер обмена пуст",
        "Selection updated" => "Выбор обновлён",
        "Canvas selection cleared" => "Выбор на холсте снят",
        "Canvas fit" => "Масштаб холста подогнан",
        "Source fit" => "Масштаб источника подогнан",
        "Back to composition" => "Возврат к композиции",
        "Source select tool active" => "Включён инструмент выбора источника",
        "Source clip selected — enter part name and comment" => {
            "Фрагмент источника выбран — укажите имя части и комментарий"
        }
        "Source clip expects x,y,w,h pixel values" => {
            "Для фрагмента источника укажите пиксельные значения x,y,w,h"
        }
        "Source clip geometry updated" => "Геометрия фрагмента источника обновлена",
        "Source clip gesture cancelled" => "Выделение фрагмента источника отменено",
        "Source clip and part metadata updated" => {
            "Фрагмент источника и метаданные части обновлены"
        }
        "Source image is missing" => "Исходное изображение отсутствует",
        "Source image dimensions are unavailable" => "Размеры исходного изображения недоступны",
        "Source clip cancelled" => "Создание фрагмента источника отменено",
        "Select a resource before adding a part" => "Перед добавлением части выберите ресурс",
        "Part name is required" => "Укажите имя части",
        "Part names must be unique within the group" => {
            "Имена частей в группе не должны повторяться"
        }
        "Part metadata saved" => "Метаданные части сохранены",
        "Part metadata updated" => "Метаданные части обновлены",
        "Part created from source clip" => "Часть создана из фрагмента источника",
        "Select a part first" => "Сначала выберите часть",
        "Part name and source are required" => "Укажите имя части и источник",
        "Part source must reference an existing image" => {
            "Источник части должен указывать на существующее изображение"
        }
        "Part deleted" => "Часть удалена",
        "Part order updated" => "Порядок частей обновлён",
        "New document tab ready" => "Новая вкладка документа готова",
        "Document tab closed" => "Вкладка документа закрыта",
        "Close cancelled" => "Закрытие отменено",
        "Save did not complete; document remains open" => {
            "Сохранение не завершено; документ остаётся открытым"
        }
        "Other document tabs closed" => "Другие вкладки документов закрыты",
        "Choose an image path first" => "Сначала укажите путь к изображению",
        "Image added; enter a name and comment, then Update" => {
            "Изображение добавлено; укажите имя и комментарий, затем нажмите «Обновить»"
        }
        "Select an image to remove" => "Выберите изображение для удаления",
        "Remove parts that use this resource first" => {
            "Сначала удалите части, использующие этот ресурс"
        }
        "Image removed" => "Изображение удалено",
        "Select an image to update" => "Выберите изображение для обновления",
        "Image name cannot be empty" => "Имя изображения не может быть пустым",
        "Image names must be unique" => "Имена изображений не должны повторяться",
        "Image metadata updated" => "Метаданные изображения обновлены",
        "Finish or cancel the current edit before saving" => {
            "Перед сохранением завершите или отмените текущее редактирование"
        }
        "Enter a .glyf path first" => "Сначала укажите путь к файлу .glyf",
        "Image selected" => "Изображение выбрано",
        "Part selected" => "Часть выбрана",
        "Annotation selected" => "Аннотация выбрана",
        "Select a resource before opening Source mode" => {
            "Перед открытием режима источника выберите ресурс"
        }
        "Point ready — enter a comment, then Add" => {
            "Точка готова — введите комментарий и нажмите «Добавить»"
        }
        "Region drag is too small" => "Выделенная область слишком мала",
        "Region ready — enter a comment, then Add" => {
            "Область готова — введите комментарий и нажмите «Добавить»"
        }
        "Annotation comment cannot be empty" => "Комментарий к аннотации не может быть пустым",
        "Annotation saved" => "Аннотация сохранена",
        "Annotation cancelled" => "Создание аннотации отменено",
        "Select an annotation first" => "Сначала выберите аннотацию",
        "Annotation deleted" => "Аннотация удалена",
        "Point moved" => "Точка перемещена",
        "Region geometry updated" => "Геометрия области обновлена",
        "Part geometry updated" => "Геометрия части обновлена",
        "Canvas 100% · 1 source px ≈ 1 logical px" => {
            "Холст 100% · 1 пиксель источника ≈ 1 логический пиксель"
        }
        "Source 100% · 1 source px ≈ 1 logical px" => {
            "Источник 100% · 1 пиксель источника ≈ 1 логический пиксель"
        }
        "Supported images: PNG, JPG, JPEG, WEBP, BMP, ICO" => {
            "Поддерживаемые форматы: PNG, JPG, JPEG, WEBP, BMP, ICO"
        }
        "Annotation copied" => "Аннотация скопирована",
        "Annotation cut" => "Аннотация вырезана",
        "Annotation no longer exists" => "Аннотация больше не существует",
        "Annotation pasted" => "Аннотация вставлена",
        "Edit Clip — move or resize, then confirm" => "Измените фрагмент и подтвердите действие",
        "Editing annotation comment" => "Изменение комментария к аннотации",
        "Editing part name and comment" => "Изменение имени и комментария части",
        "Finish or cancel the current annotation first" => "Сначала завершите или отмените текущую аннотацию",
        "Finish or cancel the current edit first" => "Сначала завершите или отмените текущее редактирование",
        "Part copied" => "Часть скопирована",
        "Part cut" => "Часть вырезана",
        "Part pasted" => "Часть вставлена",
        "Part source image is missing" => "Исходное изображение части отсутствует",
        "Region expects rel1.x,rel1.y,rel2.x,rel2.y values" => "Для области укажите rel1.x, rel1.y, rel2.x и rel2.y",
        "Region rectangle is empty after normalization" => "После нормализации область оказалась пустой",
        "Select a part with a source clip first" => "Сначала выберите часть с фрагментом источника",
        "Select an image before setting a source clip" => "Перед выбором фрагмента источника выберите изображение",
        "Selected part has no source clip" => "У выбранной части нет фрагмента источника",
        "Semantic region geometry updated" => "Геометрия области обновлена",
        "Semantic source clip geometry applied" => "Геометрия фрагмента источника применена",
        "Source clip selected" => "Фрагмент источника выбран",
        "Unsaved changes — save before Exit" => "Есть несохранённые изменения — сохраните документ перед выходом",
        "Unsaved document — choose Save, Don't Save, or Cancel" => "Документ не сохранён — выберите «Сохранить», «Не сохранять» или «Отмена»",
        "Unsaved changes — press Open again to discard, or Save first" => "Есть несохранённые изменения — нажмите «Открыть» ещё раз, чтобы отбросить их, или сначала сохраните документ",
        "Unsaved changes — close again to discard, or Save the document" => "Есть несохранённые изменения — закройте ещё раз, чтобы отбросить их, или сохраните документ",
        _ => "",
    };
    if !exact.is_empty() {
        return exact.to_owned();
    }

    if let Some(paths) = message.strip_prefix("Saved ") {
        if let Some((glyf, glyp)) = paths.rsplit_once(" and ") {
            return format!("Сохранены {glyf} и {glyp}");
        }
    }
    for (prefix, localized) in [("Undo ", "Отменено: "), ("Redo ", "Повторено: ")] {
        if let Some(action) = message.strip_prefix(prefix) {
            return format!("{localized}{}", localized_history_action(action));
        }
    }

    for (prefix, localized) in [
        ("Settings save failed:", "Не удалось сохранить настройки:"),
        ("Save failed:", "Не удалось сохранить:"),
        ("Open failed:", "Не удалось открыть:"),
        ("Pack failed:", "Не удалось упаковать:"),
        (
            "Preview decode failed:",
            "Не удалось обработать изображение для предпросмотра:",
        ),
        ("Cannot read image:", "Не удалось прочитать изображение:"),
        ("Missing resource '", "Отсутствует ресурс '"),
        ("Canvas zoom ", "Масштаб холста "),
        ("Source zoom ", "Масштаб источника "),
        ("Source mode · ", "Режим источника · "),
        ("Opened ", "Открыт файл "),
        ("Saved ", "Сохранено: "),
        ("Active document: ", "Активный документ: "),
        ("Activated already-open ", "Активирован уже открытый файл "),
    ] {
        if let Some(suffix) = message.strip_prefix(prefix) {
            return format!("{localized}{suffix}");
        }
    }

    message.to_owned()
}

fn localized_history_action(action: &str) -> &str {
    match action {
        "Paste annotation" => "Вставить аннотацию",
        "Paste part" => "Вставить часть",
        "Cut annotation" => "Вырезать аннотацию",
        "Cut part" => "Вырезать часть",
        "Update region geometry" => "Изменить геометрию области",
        "Update part metadata" => "Изменить данные части",
        "Delete annotation" => "Удалить аннотацию",
        "Delete part" => "Удалить часть",
        "Move part" => "Переместить часть",
        "Add image" => "Добавить изображение",
        "Remove image" => "Удалить изображение",
        "Update image metadata" => "Изменить данные изображения",
        "Update annotation role" => "Изменить роль аннотации",
        _ => action,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_files_are_deduplicated_and_bounded() {
        let mut settings = DesktopSettings {
            recent_file_count: 2,
            ..Default::default()
        };
        settings.add_recent("a.glyf");
        settings.add_recent("b.glyf");
        settings.add_recent("a.glyf");
        assert_eq!(settings.recent_files, ["a.glyf", "b.glyf"]);
        settings.add_recent("c.glyf");
        assert_eq!(settings.recent_files, ["c.glyf", "a.glyf"]);
    }

    #[test]
    fn language_and_theme_round_trip_through_settings_text() {
        let settings = DesktopSettings {
            language: Language::Russian,
            theme: ThemeChoice::Dark,
            default_zoom: 1.25,
            restore_session: true,
            recent_file_count: 4,
            recent_files: vec!["one.glyf".into(), "two.glyf".into()],
            ..Default::default()
        };
        let restored = parse_settings(&serialize_settings(&settings)).expect("settings parse");
        assert_eq!(restored, settings);
    }

    #[test]
    fn duplicate_recent_names_are_disambiguated_without_changing_order() {
        let entries = recent_menu_entries(&[
            "C:/work/one/demo.glyf".into(),
            "C:/work/two/demo.glyf".into(),
            "C:/work/three/other.glyf".into(),
        ]);
        assert_eq!(entries[0].path, "C:/work/one/demo.glyf");
        assert_eq!(entries[0].label, "demo.glyf — one");
        assert_eq!(entries[1].label, "demo.glyf — two");
        assert_eq!(entries[2].label, "other.glyf");
    }
    #[test]
    fn clear_recent_removes_every_saved_entry() {
        let mut settings = DesktopSettings {
            recent_files: vec!["one.glyf".into(), "two.glyf".into()],
            ..Default::default()
        };
        settings.clear_recent();
        assert!(settings.recent_files.is_empty());
    }

    #[test]
    fn missing_recent_entries_are_pruned_without_a_crash() {
        let missing = env::temp_dir().join(format!(
            "glyf-editor-missing-recent-{}-{}.glyf",
            std::process::id(),
            1_234_567_u64
        ));
        assert!(!missing.exists());
        let mut settings = DesktopSettings {
            recent_files: vec![missing.display().to_string()],
            ..Default::default()
        };
        assert!(settings.prune_missing_recent());
        assert!(settings.recent_files.is_empty());
    }

    #[test]
    fn recent_menu_localization_has_english_and_russian_labels() {
        assert_eq!(text("file.open_recent", Language::English), "Open Recent");
        assert_eq!(text("file.clear_recent", Language::English), "Clear Recent");
        assert_eq!(text("file.no_recent", Language::English), "No recent files");
        assert_eq!(
            text("file.open_recent", Language::Russian),
            "Недавние файлы"
        );
        assert_eq!(
            text("file.clear_recent", Language::Russian),
            "Очистить список"
        );
        assert_eq!(
            text("file.no_recent", Language::Russian),
            "Нет недавних файлов"
        );
    }
    #[test]
    fn configured_recent_count_is_capped_to_visible_menu_capacity() {
        let restored = parse_settings("recent_file_count=999\n").expect("settings parse");
        assert_eq!(restored.recent_file_count, RECENT_MENU_CAPACITY);
    }

    #[test]
    fn appearance_and_editor_preferences_round_trip_as_settings_codes() {
        let settings = DesktopSettings {
            style_profile: StyleProfile::neon(),
            font_size: FontSizeChoice::Large,
            ui_scale: UiScaleChoice::Percent125,
            density: DensityChoice::Compact,
            panel_resources: false,
            panel_layers: true,
            panel_inspector: false,
            panel_status_bar: true,
            confirm_destructive: false,
            ..Default::default()
        };
        let encoded = serialize_settings(&settings);
        assert!(encoded.contains("style_preset=neon\n"));
        assert!(encoded.contains("style_palette=neon\n"));
        assert!(!encoded.contains(".glyf"));
        assert_eq!(parse_settings(&encoded).expect("settings parse"), settings);
    }

    #[test]
    fn dark_light_neon_and_custom_style_profiles_persist() {
        let mut light = StyleProfile::production();
        light.palette = PaletteStyle::Light;
        let mut custom = StyleProfile::neon();
        custom.shape = ShapeStyle::Angular;
        custom.surface = SurfaceStyle::Neumorphic;
        custom.motion = MotionStyle::None;
        assert!(custom.preset().is_none(), "test profile must be custom");

        for (name, profile) in [
            ("dark", StyleProfile::production()),
            ("light", light),
            ("neon", StyleProfile::neon()),
            ("custom", custom),
        ] {
            let settings = DesktopSettings {
                theme: ThemeChoice::from_style_profile(profile),
                style_profile: profile,
                ..Default::default()
            };
            let encoded = serialize_settings(&settings);
            let restored = parse_settings(&encoded).expect("settings parse");
            assert_eq!(restored.style_profile, profile, "{name} profile changed");
            assert_eq!(
                restored.theme,
                ThemeChoice::from_style_profile(profile),
                "{name} compatibility theme changed"
            );
            if name == "custom" {
                assert!(encoded.contains("style_preset=custom\n"));
            }
        }
    }

    #[test]
    fn legacy_theme_migrates_to_a_profile_without_resetting_other_settings() {
        let mut light = StyleProfile::production();
        light.palette = PaletteStyle::Light;

        for (legacy_theme, expected_profile) in [
            ("dark", StyleProfile::production()),
            ("light", light),
            ("system", StyleProfile::production()),
        ] {
            let contents = format!(
                "language=ru\ntheme={legacy_theme}\nrestore_session=true\nrecent_file_count=4\n"
            );
            let restored = parse_settings(&contents).expect("legacy settings parse");
            assert_eq!(restored.style_profile, expected_profile, "{legacy_theme}");
            assert_eq!(
                restored.theme,
                ThemeChoice::from_style_profile(expected_profile),
                "{legacy_theme} compatibility theme"
            );
            assert_eq!(restored.language, Language::Russian);
            assert!(restored.restore_session);
            assert_eq!(restored.recent_file_count, 4);
        }
    }

    #[test]
    fn explicit_neon_or_custom_profile_wins_over_conflicting_legacy_theme() {
        let neon = parse_settings(
            "theme=light\nstyle_preset=neon\nstyle_palette=neon\n",
        )
        .expect("Neon settings parse");
        assert_eq!(neon.style_profile, StyleProfile::neon());
        assert_eq!(neon.theme, ThemeChoice::Dark);

        let mut custom = StyleProfile::neon();
        custom.palette = PaletteStyle::Light;
        custom.shape = ShapeStyle::Angular;
        custom.surface = SurfaceStyle::Neumorphic;
        custom.motion = MotionStyle::None;
        let settings = DesktopSettings {
            // A stale compatibility field must not override the profile.
            theme: ThemeChoice::Dark,
            style_profile: custom,
            ..Default::default()
        };
        let encoded = serialize_settings(&settings);
        assert!(encoded.contains("theme=light\n"));
        assert!(encoded.contains("style_preset=custom\n"));
        let restored = parse_settings(&encoded).expect("custom settings parse");
        assert_eq!(restored.style_profile, custom);
        assert_eq!(restored.theme, ThemeChoice::Light);
    }

    #[test]
    fn partial_custom_profile_keeps_legacy_palette_as_its_base() {
        let restored = parse_settings(
            "theme=light\nstyle_preset=custom\nstyle_shape=angular\n",
        )
        .expect("partial settings parse");
        assert_eq!(restored.style_profile.palette, PaletteStyle::Light);
        assert_eq!(restored.style_profile.shape, ShapeStyle::Angular);
        assert_eq!(restored.theme, ThemeChoice::Light);
    }

    #[test]
    fn parser_accepts_legacy_and_future_versions_and_ignores_unknown_keys() {
        let legacy = parse_settings("language=ru\nrestore_session=true\n").expect("legacy config");
        assert_eq!(legacy.language, Language::Russian);
        assert!(legacy.restore_session);
        let future = parse_settings("version=999\nfont_size=small\nfuture_option=hello\n")
            .expect("future config");
        assert_eq!(future.font_size, FontSizeChoice::Small);
    }

    #[test]
    fn malformed_configuration_uses_default_fallback() {
        let loaded = parse_settings("default_zoom=not-a-number\n").unwrap_or_default();
        assert_eq!(loaded, DesktopSettings::default());
        assert!(parse_settings("this is not a settings file").is_err());
    }

    #[test]
    fn escaped_recent_file_delimiters_round_trip() {
        let settings = DesktopSettings {
            recent_files: vec![r"C:\work\one|two.glyf".into(), "line\nbreak.glyf".into()],
            ..Default::default()
        };
        assert_eq!(
            parse_settings(&serialize_settings(&settings))
                .unwrap()
                .recent_files,
            settings.recent_files
        );
    }

    #[test]
    fn dropdown_choices_expose_codes_labels_and_complete_options() {
        assert_eq!(
            FontSizeChoice::ALL.map(FontSizeChoice::code),
            ["small", "normal", "large"]
        );
        assert_eq!(
            UiScaleChoice::ALL.map(UiScaleChoice::code),
            ["100", "110", "125", "150"]
        );
        assert_eq!(
            DensityChoice::ALL.map(DensityChoice::code),
            ["compact", "comfortable"]
        );
        assert!(
            FontSizeChoice::ALL
                .iter()
                .all(|choice| !choice.label().is_empty())
        );
        assert!(
            UiScaleChoice::ALL
                .iter()
                .all(|choice| !choice.label().is_empty())
        );
        assert!(
            DensityChoice::ALL
                .iter()
                .all(|choice| !choice.label().is_empty())
        );
    }

    #[test]
    fn typography_and_spacing_tokens_follow_size_scale_and_density() {
        let large = DesktopSettings {
            font_size: FontSizeChoice::Large,
            ui_scale: UiScaleChoice::Percent150,
            ..Default::default()
        };
        let compact = DesktopSettings {
            ui_scale: UiScaleChoice::Percent110,
            density: DensityChoice::Compact,
            ..Default::default()
        };
        let comfortable = DesktopSettings {
            ui_scale: UiScaleChoice::Percent110,
            density: DensityChoice::Comfortable,
            ..Default::default()
        };
        assert!(
            large.typography_tokens().interface
                > DesktopSettings::default().typography_tokens().interface
        );
        assert!(
            compact.spacing_tokens().panel_padding < comfortable.spacing_tokens().panel_padding
        );
    }

    #[test]
    fn all_requested_settings_labels_are_localized() {
        let keys = [
            "settings.general",
            "settings.appearance",
            "settings.editor",
            "settings.files",
            "settings.accessibility",
            "settings.language",
            "settings.restore_session",
            "settings.recent_count",
            "settings.preset",
            "settings.palette",
            "settings.shape",
            "settings.surface",
            "settings.motion",
            "settings.font_size",
            "settings.ui_scale",
            "settings.density",
            "settings.default_zoom",
            "settings.resources",
            "settings.layers",
            "settings.inspector",
            "settings.status_bar",
            "settings.confirm_destructive",
            "settings.reset_appearance",
            "settings.reset_all",
            "settings.close",
            "settings.accent",
            "settings.reduced_motion",
            "settings.high_contrast",
        ];
        for key in keys {
            assert!(
                !text(key, Language::English).is_empty(),
                "missing English label: {key}"
            );
            assert!(
                !text(key, Language::Russian).is_empty(),
                "missing Russian label: {key}"
            );
        }
    }

    #[test]
    fn all_editor_labels_are_localized_in_english_and_russian() {
        let keys = [
            "editor.images_heading",
            "editor.no_images_yet",
            "editor.no_images",
            "editor.image_path_placeholder",
            "editor.add_image",
            "editor.remove_image",
            "editor.metadata_heading",
            "editor.image_name_placeholder",
            "editor.source_label",
            "editor.resource_comment_placeholder",
            "editor.update_resource",
            "editor.open_source",
            "editor.layers_heading",
            "editor.no_parts_yet",
            "editor.no_parts",
            "editor.add_part",
            "editor.add_clip",
            "editor.delete_part",
            "editor.edit_source_crop",
            "editor.up",
            "editor.down",
            "editor.back",
            "editor.front",
            "editor.layers_hint",
            "editor.annotations_heading",
            "editor.no_annotations_yet",
            "editor.no_annotations",
            "editor.edit_annotation",
            "editor.delete_annotation",
            "editor.annotations_hint",
            "editor.no_selection",
            "editor.role_label",
            "editor.coordinates_label",
            "editor.comment_label",
            "editor.add_annotation",
            "editor.annotation_prompt",
            "editor.cancel",
            "editor.done",
            "editor.preview_empty",
            "editor.composition",
            "editor.source_mode",
            "editor.saved",
            "editor.unsaved",
            "editor.ready",
        ];
        for key in keys {
            assert!(
                !text(key, Language::English).is_empty(),
                "missing English label: {key}"
            );
            assert!(
                !text(key, Language::Russian).is_empty(),
                "missing Russian label: {key}"
            );
        }
    }

    #[test]
    fn localized_status_translates_exact_and_dynamic_messages() {
        assert_eq!(
            localized_status("Canvas selection cleared", Language::Russian),
            "Выбор на холсте снят"
        );
        assert_eq!(
            localized_status("Image removed", Language::Russian),
            "Изображение удалено"
        );
        assert_eq!(
            localized_status("Save failed: permission denied", Language::Russian),
            "Не удалось сохранить: permission denied"
        );
        assert_eq!(
            localized_status("Canvas zoom 125%", Language::Russian),
            "Масштаб холста 125%"
        );
        assert_eq!(
            localized_status("Undo Add image", Language::Russian),
            "Отменено: Добавить изображение"
        );
        assert_eq!(
            localized_status("Saved demo.glyf and demo.glyp", Language::Russian),
            "Сохранены demo.glyf и demo.glyp"
        );
        assert_eq!(
            localized_status("Unsaved changes — save before Exit", Language::Russian),
            "Есть несохранённые изменения — сохраните документ перед выходом"
        );
        assert_eq!(
            localized_status("unrecognized status", Language::Russian),
            "unrecognized status"
        );
        assert_eq!(
            localized_status("Image removed", Language::English),
            "Image removed"
        );
    }
}
