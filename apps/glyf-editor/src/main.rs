use designing_makepad::{
    DesktopMenu, MenuChainState, MenuRect, MotionStyle, PaletteStyle, ShapeStyle, StylePreset,
    StyleProfile, StyleTuning, SurfaceStyle, place_submenu,
};
use designing_makepad::style::ButtonVariant;
use glyf_core::{
    GlyfAnnotation, GlyfAnnotationGeometry, GlyfClip, GlyfDocument, GlyfGroup, GlyfImage, GlyfPart,
    GlyfPoint, parse_file, serialize_file,
};
use glyf_pack::{GlypReader, GlypWriter};
use makepad_widgets::makepad_platform::event::{MouseButton, TouchState};
use makepad_widgets::*;
use std::{
    fs,
    path::{Path, PathBuf},
};
#[cfg(all(windows, feature = "toolkit-uia"))]
use std::sync::Arc;

mod clip_popup;
mod desktop;
mod file_dialog;
mod menu_state;
mod overlay_host;
mod shell_style;
mod source;
mod spatial;
mod tab_strip;

use clip_popup::ClipPopup;
use desktop::{
    DensityChoice, DesktopSettings, FontSizeChoice, Language, RECENT_MENU_CAPACITY, ThemeChoice,
    UiScaleChoice,
};
use file_dialog::FileDialogKind;
use menu_state::{MenuState, MenuStateSnapshot};
use overlay_host::{OverlayHost, OverlayHostAction};
use source::{SourceCanvas, SourceCanvasAction, SourceTool};
use spatial::{
    AnnotationCanvasAction, CanvasEditAction, CanvasHoverAction, CanvasTool, CanvasViewportAction,
    MAX_VIEW_ZOOM, MIN_VIEW_ZOOM, SpatialCanvas, SpatialCanvasAction, clamp_source_clip,
    image_size_from_data, normalize_region, one_to_one_zoom, relative_to_canvas,
    source_rect_to_screen, zoom_view_around, zoomed_content_rect,
};
use tab_strip::{DocumentTabFrame, DocumentTabStrip, DocumentTabStripAction};

app_main!(App);

const WINDOW_TITLE: &str = "GLYF Editor";

// Cohesive editor adapters; App remains the single state/lifecycle owner.
mod model;
mod helpers;
mod documents;
mod project_io;
mod history;
mod settings;
mod input;
mod canvas;
mod source_clip;
mod parts;
mod annotations;
mod shell;
mod events;
mod ui;
#[cfg(all(windows, feature = "toolkit-uia"))]
mod accessibility;
#[cfg(windows)]
mod platform;

use model::*;
use helpers::*;
#[cfg(windows)]
use platform::*;

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    images: Vec<EditorImage>,
    #[rust]
    groups: Vec<GlyfGroup>,
    #[rust]
    annotations: Vec<GlyfAnnotation>,
    #[rust]
    selected: Option<usize>,
    #[rust]
    selected_annotation: Option<usize>,
    #[rust]
    tool: CanvasTool,
    #[rust]
    canvas_zoom: f64,
    #[rust]
    canvas_pan: DVec2,
    #[rust]
    hovered_annotation: Option<usize>,
    #[rust]
    hovered_part: Option<usize>,
    #[rust]
    source_mode: bool,
    #[rust]
    source_tool: SourceTool,
    #[rust]
    source_zoom: f64,
    #[rust]
    source_pan: DVec2,
    #[rust]
    source_clip: Option<GlyfClip>,
    #[rust]
    pending_clip: Option<PendingClip>,
    #[rust]
    pending_part_popup: Option<PendingPartPopup>,
    #[rust]
    pending_annotation: Option<PendingAnnotation>,
    #[rust]
    annotation_draft: String,
    #[rust]
    annotation_popup_focus_on_next_draw: bool,
    #[rust]
    selected_part: Option<usize>,
    #[rust]
    project_path: String,
    #[rust]
    status_text: String,
    #[rust]
    dirty: bool,
    #[rust]
    atlases_dirty: bool,
    #[rust]
    close_confirm_armed: bool,
    #[rust]
    pending_replace: Option<ReplaceIntent>,
    #[rust]
    settings: DesktopSettings,
    #[rust]
    settings_category: SettingsCategory,
    #[rust]
    style_profile: StyleProfile,
    #[rust]
    tabs: Vec<DocumentTab>,
    #[rust]
    active_tab: usize,
    #[rust]
    next_tab_id: u64,
    #[rust]
    pending_tab_close: Option<u64>,
    #[rust]
    menu_open: Option<String>,
    #[rust]
    menu_chain: MenuChainState,
    #[rust]
    context_menu_open: bool,
    #[rust]
    overlay_pos: Option<DVec2>,
    #[rust]
    panel_resources: bool,
    #[rust]
    panel_layers: bool,
    #[rust]
    clipboard: Option<ClipboardPayload>,
    #[rust]
    undo_stack: Vec<HistoryEntry>,
    #[rust]
    redo_stack: Vec<HistoryEntry>,
    #[rust]
    gesture_history_before: Option<DocumentSnapshot>,
    #[rust]
    hovered_toolbar_tooltip: Option<String>,
}
