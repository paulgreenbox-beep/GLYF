//! Undo/redo history and application clipboard commands.

use super::*;

impl App {
    pub(crate) fn begin_gesture_history(&mut self) {
        if self.gesture_history_before.is_none() {
            self.gesture_history_before = Some(self.capture_document_snapshot());
        }
    }

    pub(crate) fn finish_gesture_history(&mut self, label: impl Into<String>) {
        if let Some(before) = self.gesture_history_before.take() {
            self.record_history(label, before);
        }
    }
    pub(crate) fn record_history(&mut self, label: impl Into<String>, before: DocumentSnapshot) {
        let after = self.capture_document_snapshot();
        if before == after {
            return;
        }
        self.undo_stack.push(HistoryEntry {
            label: label.into(),
            before,
            after,
        });
        self.redo_stack.clear();
        self.sync_active_tab_snapshot();
    }

    pub(crate) fn undo(&mut self, cx: &mut Cx) {
        let Some(entry) = self.undo_stack.pop() else {
            self.set_status(cx, "Nothing to undo");
            return;
        };
        let current = self.capture_document_snapshot();
        self.apply_document_snapshot(entry.before.clone());
        self.redo_stack.push(HistoryEntry {
            label: entry.label.clone(),
            before: entry.before,
            after: current,
        });
        self.set_dirty(cx, false);
        self.refresh(cx);
        self.set_status(cx, format!("Undo {}", entry.label));
        self.sync_active_tab_snapshot();
    }

    pub(crate) fn redo(&mut self, cx: &mut Cx) {
        let Some(entry) = self.redo_stack.pop() else {
            self.set_status(cx, "Nothing to redo");
            return;
        };
        let current = self.capture_document_snapshot();
        self.apply_document_snapshot(entry.after.clone());
        self.undo_stack.push(HistoryEntry {
            label: entry.label.clone(),
            before: current,
            after: entry.after,
        });
        self.set_dirty(cx, false);
        self.refresh(cx);
        self.set_status(cx, format!("Redo {}", entry.label));
        self.sync_active_tab_snapshot();
    }

    pub(crate) fn copy_selection(&mut self, cx: &mut Cx) {
        if let Some(index) = self.selected_annotation {
            if let Some(annotation) = self.annotations.get(index).cloned() {
                cx.copy_to_clipboard(&annotation.comment);
                self.clipboard = Some(ClipboardPayload::Annotation(annotation));
                self.set_status(cx, "Annotation copied");
                return;
            }
        }
        if let Some(index) = self.selected_part {
            if let Some(part) = self.current_parts().get(index).cloned() {
                cx.copy_to_clipboard(&part.comment);
                self.clipboard = Some(ClipboardPayload::Part(part));
                self.set_status(cx, "Part copied");
                return;
            }
        }
        self.set_status(cx, "Select an annotation or part first");
    }

    pub(crate) fn paste_selection(&mut self, cx: &mut Cx) {
        let Some(payload) = self.clipboard.clone() else {
            self.set_status(cx, "Clipboard is empty");
            return;
        };
        let before = self.capture_document_snapshot();
        match payload {
            ClipboardPayload::Annotation(mut annotation) => {
                annotation.name =
                    unique_annotation_name(&self.annotations, &format!("{}_copy", annotation.name));
                annotation.geometry = match annotation.geometry {
                    GlyfAnnotationGeometry::Point { rel } => GlyfAnnotationGeometry::Point {
                        rel: GlyfPoint {
                            x: (rel.x + 0.03).min(0.98),
                            y: (rel.y + 0.03).min(0.98),
                        },
                    },
                    GlyfAnnotationGeometry::Region { rel1, rel2 } => {
                        GlyfAnnotationGeometry::Region {
                            rel1: GlyfPoint {
                                x: (rel1.x + 0.03).min(0.98),
                                y: (rel1.y + 0.03).min(0.98),
                            },
                            rel2: GlyfPoint {
                                x: (rel2.x + 0.03).min(0.99),
                                y: (rel2.y + 0.03).min(0.99),
                            },
                        }
                    }
                };
                self.annotations.push(annotation);
                self.selected_annotation = Some(self.annotations.len() - 1);
                self.selected_part = None;
                self.mark_dirty(cx);
                self.sync_spatial(cx);

                self.record_history("Paste annotation", before);
                self.set_status(cx, "Annotation pasted");
            }
            ClipboardPayload::Part(mut part) => {
                part.name = unique_part_name(self.current_parts(), &format!("{}_copy", part.name));
                part.rel1.x = (part.rel1.x + 0.03).min(0.98);
                part.rel1.y = (part.rel1.y + 0.03).min(0.98);
                part.rel2.x = (part.rel2.x + 0.03).min(0.99);
                part.rel2.y = (part.rel2.y + 0.03).min(0.99);
                let before = self.capture_document_snapshot();
                let parts = self.current_parts_mut();
                parts.push(part);
                self.selected_part = Some(parts.len() - 1);
                self.selected_annotation = None;
                self.mark_dirty(cx);
                self.sync_spatial(cx);

                self.record_history("Paste part", before);
                self.set_status(cx, "Part pasted");
            }
        }
    }

    pub(crate) fn duplicate_selection(&mut self, cx: &mut Cx) {
        self.copy_selection(cx);
        if self.clipboard.is_some() {
            self.paste_selection(cx);
        }
    }

    pub(crate) fn cut_selection(&mut self, cx: &mut Cx) {
        let before = self.capture_document_snapshot();
        self.copy_selection(cx);
        if let Some(index) = self.selected_annotation {
            if index < self.annotations.len() {
                self.annotations.remove(index);
                self.selected_annotation = None;
                self.mark_dirty(cx);
                self.sync_spatial(cx);

                self.record_history("Cut annotation", before);
                self.set_status(cx, "Annotation cut");
            }
        } else if let Some(index) = self.selected_part {
            let before = self.capture_document_snapshot();
            let parts = self.current_parts_mut();
            if index < parts.len() {
                parts.remove(index);
                self.selected_part = None;
                self.mark_dirty(cx);
                self.sync_spatial(cx);

                self.record_history("Cut part", before);
                self.set_status(cx, "Part cut");
            }
        }
    }

    pub(crate) fn select_all(&mut self, cx: &mut Cx) {
        if !self.annotations.is_empty() {
            self.selected_annotation = Some(0);
            self.selected_part = None;
        } else if !self.current_parts().is_empty() {
            self.selected_part = Some(0);
            self.selected_annotation = None;
        }
        self.sync_spatial(cx);

        self.set_status(cx, "Selection updated");
    }
}
