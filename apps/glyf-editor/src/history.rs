//! Undo/redo history and application clipboard commands.

use super::*;

// Retain the current path and check content against the tab's saved checkpoint.
fn move_history(
    current: DocumentSnapshot,
    from: &mut Vec<HistoryEntry>,
    to: &mut Vec<HistoryEntry>,
    saved: Option<&DocumentSnapshot>,
    undo: bool,
) -> Option<(DocumentSnapshot, String)> {
    let entry = from.pop()?;
    let mut target = if undo { entry.before.clone() } else { entry.after.clone() };
    target.project_path = current.project_path.clone();
    target.dirty = saved.map_or(true, |checkpoint| !target.same_saved_content(checkpoint));
    let label = entry.label.clone();
    to.push(if undo {
        HistoryEntry { label: entry.label, before: entry.before, after: current }
    } else {
        HistoryEntry { label: entry.label, before: current, after: entry.after }
    });
    Some((target, label))
}

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
        self.restore_history(cx, true);
    }

    pub(crate) fn redo(&mut self, cx: &mut Cx) {
        self.restore_history(cx, false);
    }

    fn restore_history(&mut self, cx: &mut Cx, undo: bool) {
        if self.has_pending_work() || self.canvas_interaction_active(cx) {
            self.set_status(cx, "Finish or cancel the current edit before undo or redo");
            return;
        }
        let current = self.capture_document_snapshot();
        let saved = self.tabs.get(self.active_tab).and_then(|tab| tab.saved_snapshot.as_ref());
        let result = if undo {
            move_history(current, &mut self.undo_stack, &mut self.redo_stack, saved, true)
        } else {
            move_history(current, &mut self.redo_stack, &mut self.undo_stack, saved, false)
        };
        let Some((snapshot, label)) = result else {
            self.set_status(cx, if undo { "Nothing to undo" } else { "Nothing to redo" });
            return;
        };
        let dirty = snapshot.dirty;
        let preview_changed = self.apply_document_snapshot(snapshot);
        self.set_dirty(cx, dirty);
        if preview_changed { self.load_preview(cx); }
        self.refresh(cx);
        self.set_status(cx, format!("{} {}", if undo { "Undo" } else { "Redo" }, label));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tab(path: &str) -> DocumentTab {
        let mut snapshot = DocumentSnapshot::default();
        snapshot.project_path = path.into();
        DocumentTab {
            id: 1,
            title: path.into(),
            path: path.into(),
            dirty: false,
            saved_snapshot: Some(snapshot.clone()),
            snapshot,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    fn edit(tab: &mut DocumentTab, comment: &str) {
        let before = tab.snapshot.clone();
        if tab.snapshot.images.is_empty() {
            tab.snapshot.images.push(EditorImage {
                name: "image".into(),
                source: "image.png".into(),
                comment: comment.into(),
                bytes: vec![1, 2, 3].into(),
                pixel_size: None,
            });
        } else {
            tab.snapshot.images[0].comment = comment.into();
        }
        tab.snapshot.dirty = true;
        tab.undo_stack.push(HistoryEntry {
            label: "Edit".into(),
            before,
            after: tab.snapshot.clone(),
        });
        tab.redo_stack.clear();
    }

    fn step(tab: &mut DocumentTab, undo: bool) {
        let current = tab.snapshot.clone();
        let (from, to) = if undo {
            (&mut tab.undo_stack, &mut tab.redo_stack)
        } else {
            (&mut tab.redo_stack, &mut tab.undo_stack)
        };
        tab.snapshot = move_history(current, from, to, tab.saved_snapshot.as_ref(), undo)
            .expect("history entry").0;
    }

    fn save(tab: &mut DocumentTab, path: &str) {
        tab.snapshot.project_path = path.into();
        tab.snapshot.dirty = false;
        tab.saved_snapshot = Some(tab.snapshot.clone());
    }

    #[test]
    fn two_unsaved_edits_undo_redo_and_branch() {
        let mut doc = tab("first.glyf");
        edit(&mut doc, "one");
        edit(&mut doc, "two");
        step(&mut doc, true);
        assert_eq!(doc.snapshot.images[0].comment, "one");
        assert!(doc.snapshot.dirty);
        step(&mut doc, false);
        assert_eq!(doc.snapshot.images[0].comment, "two");
        assert!(doc.snapshot.dirty);
        step(&mut doc, true);
        step(&mut doc, true);
        assert!(!doc.snapshot.dirty);
        step(&mut doc, false);
        assert!(doc.snapshot.dirty);
        edit(&mut doc, "branch");
        assert!(doc.redo_stack.is_empty());
        step(&mut doc, true);
        assert!(doc.snapshot.dirty);
    }

    #[test]
    fn save_between_edits_and_save_as_preserve_checkpoint_and_path() {
        let mut doc = tab("old.glyf");
        edit(&mut doc, "one");
        save(&mut doc, "old.glyf");
        edit(&mut doc, "two");
        step(&mut doc, true);
        assert!(!doc.snapshot.dirty);
        step(&mut doc, true);
        assert!(doc.snapshot.dirty);
        step(&mut doc, false);
        assert!(!doc.snapshot.dirty);
        step(&mut doc, false);
        assert!(doc.snapshot.dirty);
        save(&mut doc, "new.glyf");
        step(&mut doc, true);
        assert!(doc.snapshot.dirty);
        assert_eq!(doc.snapshot.project_path, "new.glyf");
        step(&mut doc, false);
        assert!(!doc.snapshot.dirty);
        assert_eq!(doc.snapshot.project_path, "new.glyf");
        step(&mut doc, true);
        save(&mut doc, "third.glyf");
        step(&mut doc, false);
        assert!(doc.snapshot.dirty);
        assert_eq!(doc.snapshot.project_path, "third.glyf");
        step(&mut doc, true);
        assert!(!doc.snapshot.dirty);
    }

    #[test]
    fn checkpoints_and_paths_are_isolated_when_switching_tabs() {
        let mut tabs = [tab("a.glyf"), tab("b.glyf")];
        edit(&mut tabs[0], "a1");
        save(&mut tabs[0], "a-new.glyf");
        edit(&mut tabs[0], "a2");
        edit(&mut tabs[1], "b1");
        step(&mut tabs[0], true);
        assert!(!tabs[0].snapshot.dirty);
        assert_eq!(tabs[0].snapshot.project_path, "a-new.glyf");
        assert!(tabs[1].snapshot.dirty);
        step(&mut tabs[1], true);
        assert!(!tabs[1].snapshot.dirty);
        assert_eq!(tabs[1].snapshot.project_path, "b.glyf");
        step(&mut tabs[0], false);
        assert!(tabs[0].snapshot.dirty);
        assert!(!tabs[1].snapshot.dirty);
    }

    #[test]
    fn checkpoint_ignores_view_state_but_includes_resource_bytes() {
        let mut doc = tab("test.glyf");
        edit(&mut doc, "one");
        save(&mut doc, "test.glyf");
        let checkpoint = doc.saved_snapshot.as_ref().unwrap();
        let mut other = checkpoint.clone();
        other.canvas_zoom = 2.0;
        other.project_path = "elsewhere.glyf".into();
        other.dirty = true;
        assert!(other.same_saved_content(checkpoint));
        other.images[0].bytes = vec![9].into();
        assert!(!other.same_saved_content(checkpoint));
        assert!(move_history(other, &mut doc.undo_stack, &mut doc.redo_stack, None, true)
            .unwrap().0.dirty);
    }
}
