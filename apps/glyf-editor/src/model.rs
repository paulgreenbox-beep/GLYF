//! Editor-owned document state. Payloads stay shared across tabs and history.

use glyf_core::{GlyfAnnotation, GlyfClip, GlyfGroup, GlyfPart};
use makepad_widgets::DVec2;
use std::sync::Arc;
use crate::{source::SourceTool, spatial::CanvasTool};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EditorImage {
    pub(crate) name: String,
    pub(crate) source: String,
    pub(crate) comment: String,
    pub(crate) bytes: Arc<[u8]>,
    // Image bytes are immutable after import; dimensions must not be decoded
    // again by viewport, layout, or accessibility updates on the UI thread.
    pub(crate) pixel_size: Option<DVec2>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DocumentSnapshot {
    pub(crate) images: Vec<EditorImage>,
    pub(crate) groups: Vec<GlyfGroup>,
    pub(crate) annotations: Vec<GlyfAnnotation>,
    pub(crate) selected: Option<usize>,
    pub(crate) selected_annotation: Option<usize>,
    pub(crate) tool: CanvasTool,
    pub(crate) canvas_zoom: f64,
    pub(crate) canvas_pan: DVec2,
    pub(crate) hovered_annotation: Option<usize>,
    pub(crate) hovered_part: Option<usize>,
    pub(crate) source_mode: bool,
    pub(crate) source_tool: SourceTool,
    pub(crate) source_zoom: f64,
    pub(crate) source_pan: DVec2,
    pub(crate) source_clip: Option<GlyfClip>,
    pub(crate) selected_part: Option<usize>,
    pub(crate) project_path: String,
    pub(crate) dirty: bool,
}

impl Default for DocumentSnapshot {
    fn default() -> Self {
        Self {
            images: Vec::new(),
            groups: Vec::new(),
            annotations: Vec::new(),
            selected: None,
            selected_annotation: None,
            tool: CanvasTool::Select,
            canvas_zoom: 1.0,
            canvas_pan: DVec2::default(),
            hovered_annotation: None,
            hovered_part: None,
            source_mode: false,
            source_tool: SourceTool::Select,
            source_zoom: 1.0,
            source_pan: DVec2::default(),
            source_clip: None,
            selected_part: None,
            project_path: String::new(),
            dirty: false,
        }
    }
}

impl DocumentSnapshot {
    pub(crate) fn same_atlas_sources(left: &[EditorImage], right: &[EditorImage]) -> bool {
        left.len() == right.len() && left.iter().zip(right).all(|(a, b)| {
            a.name == b.name && Arc::ptr_eq(&a.bytes, &b.bytes)
        })
    }

    // Compare persisted content, excluding view state, cached dimensions and path.
    pub(crate) fn same_saved_content(&self, other: &Self) -> bool {
        self.images.len() == other.images.len()
            && self.images.iter().zip(&other.images).all(|(a, b)| {
                a.name == b.name && a.source == b.source && a.comment == b.comment
                    && (Arc::ptr_eq(&a.bytes, &b.bytes) || a.bytes == b.bytes)
            })
            && self.groups == other.groups
            && self.annotations == other.annotations
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DocumentTab {
    pub(crate) id: u64,
    pub(crate) title: String,
    pub(crate) path: String,
    pub(crate) dirty: bool,
    pub(crate) snapshot: DocumentSnapshot,
    pub(crate) saved_snapshot: Option<DocumentSnapshot>,
    pub(crate) undo_stack: Vec<HistoryEntry>,
    pub(crate) redo_stack: Vec<HistoryEntry>,
}
#[derive(Clone, Debug)]
pub(crate) enum ClipboardPayload {
    Annotation(GlyfAnnotation),
    Part(GlyfPart),
}

#[derive(Clone, Debug)]
pub(crate) struct HistoryEntry {
    pub(crate) label: String,
    pub(crate) before: DocumentSnapshot,
    pub(crate) after: DocumentSnapshot,
}

#[derive(Clone, Debug)]
pub(crate) enum PendingAnnotation {
    /// The shape is visible on the canvas, but is not yet part of the
    /// serializable document or undo history.
    Create { annotation: GlyfAnnotation },
    /// Editing only owns a draft comment; the stored annotation stays intact
    /// until Apply.
    Edit { index: usize },
}

#[derive(Clone, Debug)]
pub(crate) struct PendingPartPopup {
    pub(crate) index: usize,
    pub(crate) original_name: String,
    pub(crate) original_comment: String,
    pub(crate) draft_name: String,
    pub(crate) draft_comment: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReplaceIntent {
    Open,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SettingsCategory {
    #[default]
    General,
    Appearance,
    Editor,
    Files,
    Accessibility,
}

#[derive(Clone, Debug)]
pub(crate) struct PendingClip {
    pub(crate) part_index: Option<usize>,
    pub(crate) source_index: usize,
    pub(crate) original_clip: Option<GlyfClip>,
    pub(crate) clip: GlyfClip,
    pub(crate) original_name: String,
    pub(crate) original_comment: String,
    pub(crate) draft_name: String,
    pub(crate) draft_comment: String,
}

#[cfg(test)]
mod snapshot_performance_tests {
    use super::*;
    use makepad_widgets::dvec2;

    fn image_snapshot() -> DocumentSnapshot {
        DocumentSnapshot {
            images: vec![EditorImage {
                name: "image".into(),
                source: "image.png".into(),
                comment: "original".into(),
                bytes: vec![42; 4 * 1024 * 1024].into(),
                pixel_size: Some(dvec2(1600.0, 900.0)),
            }],
            ..DocumentSnapshot::default()
        }
    }

    #[test]
    fn snapshots_share_image_payload_but_keep_independent_metadata() {
        let before = image_snapshot();
        let mut after = before.clone();
        after.images[0].comment = "edited".into();
        assert!(Arc::ptr_eq(&before.images[0].bytes, &after.images[0].bytes));
        assert_eq!(after.images[0].pixel_size, before.images[0].pixel_size);
        assert_eq!(before.images[0].comment, "original");
        assert_ne!(before, after);
    }

    #[test]
    fn atlas_identity_changes_only_when_image_identity_or_payload_changes() {
        let before = image_snapshot();
        let mut after = before.clone();
        after.images[0].comment = "edited metadata".into();
        assert!(DocumentSnapshot::same_atlas_sources(&before.images, &after.images));
        after.images[0].name = "renamed".into();
        assert!(!DocumentSnapshot::same_atlas_sources(&before.images, &after.images));
        after.images[0].name = before.images[0].name.clone();
        after.images[0].bytes = vec![7, 8].into();
        assert!(!DocumentSnapshot::same_atlas_sources(&before.images, &after.images));
    }

    #[test]
    fn cloned_tab_and_history_share_the_original_image_allocation() {
        let original = image_snapshot();
        let tab = DocumentTab {
            id: 1,
            title: "test".into(),
            path: "test.glyf".into(),
            dirty: false,
            snapshot: original.clone(),
            saved_snapshot: Some(original.clone()),
            undo_stack: vec![HistoryEntry {
                label: "edit".into(),
                before: original.clone(),
                after: original.clone(),
            }],
            redo_stack: Vec::new(),
        };
        let cloned = tab.clone();
        for snapshot in [
            &cloned.snapshot,
            &cloned.undo_stack[0].before,
            &cloned.undo_stack[0].after,
        ] {
            assert!(Arc::ptr_eq(&original.images[0].bytes, &snapshot.images[0].bytes));
            assert_eq!(snapshot, &original);
        }
    }
}
