//! Document snapshots, tab lifetime and unsaved-work bookkeeping.

use super::*;

impl App {
    pub(crate) fn capture_document_snapshot(&self) -> DocumentSnapshot {
        DocumentSnapshot {
            images: self.images.clone(),
            groups: self.groups.clone(),
            annotations: self.annotations.clone(),
            selected: self.selected,
            selected_annotation: self.selected_annotation,
            tool: self.tool,
            canvas_zoom: self.canvas_zoom,
            canvas_pan: self.canvas_pan,
            hovered_annotation: self.hovered_annotation,
            hovered_part: self.hovered_part,
            source_mode: self.source_mode,
            source_tool: self.source_tool,
            source_zoom: self.source_zoom,
            source_pan: self.source_pan,
            source_clip: self.source_clip,
            selected_part: self.selected_part,
            project_path: self.project_path.clone(),
            dirty: self.dirty,
        }
    }

    pub(crate) fn apply_document_snapshot(&mut self, snapshot: DocumentSnapshot) -> bool {
        let atlas_changed = !DocumentSnapshot::same_atlas_sources(&self.images, &snapshot.images);
        let preview_changed = atlas_changed || self.selected != snapshot.selected;
        self.atlases_dirty |= atlas_changed;
        self.images = snapshot.images;
        self.groups = snapshot.groups;
        self.annotations = snapshot.annotations;
        self.selected = snapshot.selected;
        self.selected_annotation = snapshot.selected_annotation;
        self.tool = snapshot.tool;
        self.canvas_zoom = snapshot.canvas_zoom;
        self.canvas_pan = snapshot.canvas_pan;
        self.hovered_annotation = snapshot.hovered_annotation;
        self.hovered_part = snapshot.hovered_part;
        self.source_mode = snapshot.source_mode;
        self.source_tool = snapshot.source_tool;
        self.source_zoom = snapshot.source_zoom;
        self.source_pan = snapshot.source_pan;
        self.source_clip = snapshot.source_clip;
        self.selected_part = snapshot.selected_part;
        self.project_path = snapshot.project_path;
        self.dirty = snapshot.dirty;
        self.pending_annotation = None;
        self.pending_clip = None;
        self.pending_part_popup = None;
        self.annotation_draft.clear();
        preview_changed
    }

    pub(crate) fn allocate_tab_id(&mut self) -> u64 {
        self.next_tab_id = self
            .next_tab_id
            .checked_add(1)
            .expect("document tab ID allocator exhausted");
        self.next_tab_id
    }

    pub(crate) fn document_tab_key(id: u64) -> LiveId {
        LiveId::from_str_num("glyf-document-tab", id)
    }

    pub(crate) fn document_tab_index_for_key(&self, key: LiveId) -> Option<usize> {
        self.tabs
            .iter()
            .position(|tab| Self::document_tab_key(tab.id) == key)
    }

    pub(crate) fn document_tab_frames(&self) -> Vec<DocumentTabFrame> {
        self.tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| DocumentTabFrame {
                key: Self::document_tab_key(tab.id),
                title: if self.settings.language == Language::Russian {
                    tab.title.strip_prefix("Untitled ")
                        .map(|number| format!("Безымянный {number}"))
                        .unwrap_or_else(|| tab.title.clone())
                } else {
                    tab.title.clone()
                },
                full_path: tab.path.clone(),
                dirty: tab.dirty,
                active: index == self.active_tab,
            })
            .collect()
    }

    pub(crate) fn sync_document_tabs(&mut self, cx: &mut Cx) {
        let frame = self.document_tab_frames();
        if let Some(mut tabs) = self
            .ui
            .widget(cx, ids!(tabs))
            .borrow_mut::<DocumentTabStrip>()
        {
            tabs.set_tabs(cx, frame);
        }
    }

    pub(crate) fn document_tab_strip_rect(&self, cx: &Cx) -> Rect {
        self.ui
            .widget(cx, ids!(tabs))
            .borrow::<DocumentTabStrip>()
            .map(|tabs| tabs.bar_rect(cx))
            .unwrap_or_else(|| self.ui.view(cx, ids!(chrome_bar)).area().rect(cx))
    }

    pub(crate) fn document_tab_rect(&self, cx: &Cx, tab: &DocumentTab) -> Option<Rect> {
        self.ui
            .widget(cx, ids!(tabs))
            .borrow::<DocumentTabStrip>()
            .and_then(|tabs| tabs.tab_rect(cx, Self::document_tab_key(tab.id)))
    }

    pub(crate) fn document_tab_close_rect(&self, cx: &Cx, tab: &DocumentTab) -> Option<Rect> {
        self.ui
            .widget(cx, ids!(tabs))
            .borrow::<DocumentTabStrip>()
            .and_then(|tabs| tabs.tab_close_rect(cx, Self::document_tab_key(tab.id)))
    }

    pub(crate) fn handle_document_tab_action(&mut self, cx: &mut Cx, action: DocumentTabStripAction) {
        match action {
            DocumentTabStripAction::Activate(key) => {
                if let Some(index) = self.document_tab_index_for_key(key) {
                    self.activate_tab(cx, index);
                }
            }
            DocumentTabStripAction::Close(key) => {
                if let Some(index) = self.document_tab_index_for_key(key) {
                    self.request_close_tab(cx, index);
                }
            }
            DocumentTabStripAction::None => {}
        }
    }

    pub(crate) fn open_tab_overflow(&mut self, cx: &mut Cx) {
        self.context_menu_open = false;
        self.menu_open = Some("window".into());
        let rect = self.ui.button(cx, ids!(tab_overflow)).area().rect(cx);
        self.overlay_pos = if rect.size.x > 0.0 && rect.size.y > 0.0 {
            Some(dvec2(rect.pos.x, rect.pos.y + rect.size.y + 4.0))
        } else {
            Some(self.shell_overlay_pos(cx, "window"))
        };
        self.sync_shell_labels(cx);
    }

    pub(crate) fn active_tab_mut(&mut self) -> Option<&mut DocumentTab> {
        self.tabs.get_mut(self.active_tab)
    }

    pub(crate) fn sync_active_tab_snapshot(&mut self) {
        let snapshot = self.capture_document_snapshot();
        let path = self.project_path.clone();
        let dirty = self.dirty;
        let active_tab = self.active_tab;
        let undo_stack = self.undo_stack.clone();
        let redo_stack = self.redo_stack.clone();
        if let Some(tab) = self.active_tab_mut() {
            tab.snapshot = snapshot;
            tab.path = path.clone();
            tab.dirty = dirty;
            tab.title = tab_title(&path, active_tab);
            tab.undo_stack = undo_stack;
            tab.redo_stack = redo_stack;
        }
    }

    pub(crate) fn ensure_active_tab(&mut self) {
        if self.tabs.is_empty() {
            let tab_id = self.allocate_tab_id();
            self.tabs.push(DocumentTab {
                id: tab_id,
                title: "Untitled 1".into(),
                path: self.project_path.clone(),
                dirty: self.dirty,
                snapshot: self.capture_document_snapshot(),
                saved_snapshot: (!self.dirty).then(|| self.capture_document_snapshot()),
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
            });
            self.active_tab = 0;
        }
    }

    pub(crate) fn activate_tab(&mut self, cx: &mut Cx, index: usize) {
        if index >= self.tabs.len() || index == self.active_tab {
            return;
        }
        if self.has_pending_work() || self.canvas_interaction_active(cx) {
            self.set_status(cx, "Finish or cancel the current edit before switching documents");
            return;
        }
        self.sync_active_tab_snapshot();
        self.active_tab = index;
        let snapshot = self.tabs[index].snapshot.clone();
        let undo_stack = self.tabs[index].undo_stack.clone();
        let redo_stack = self.tabs[index].redo_stack.clone();
        let preview_changed = self.apply_document_snapshot(snapshot);
        self.undo_stack = undo_stack;
        self.redo_stack = redo_stack;
        if preview_changed { self.load_preview(cx); }
        self.refresh(cx);
        self.set_status(cx, format!("Active document: {}", self.tabs[index].title));
    }
    pub(crate) fn next_tab(&mut self, cx: &mut Cx) {
        if self.tabs.len() > 1 {
            self.activate_tab(cx, (self.active_tab + 1) % self.tabs.len());
        }
    }

    pub(crate) fn previous_tab(&mut self, cx: &mut Cx) {
        if self.tabs.len() > 1 {
            let index = if self.active_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab - 1
            };
            self.activate_tab(cx, index);
        }
    }
    pub(crate) fn checkpoint_saved_document(&mut self) {
        let snapshot = self.capture_document_snapshot();
        if let Some(tab) = self.active_tab_mut() {
            tab.saved_snapshot = Some(snapshot);
        }
    }

    pub(crate) fn set_dirty(&mut self, cx: &mut Cx, dirty: bool) {
        self.dirty = dirty;
        self.close_confirm_armed = false;
        self.pending_replace = None;
        self.ui
            .label(cx, ids!(chrome_title))
            .set_text(cx, if dirty { "GLYF Editor *" } else { WINDOW_TITLE });
        self.ui
            .label(cx, ids!(dirty_badge))
            .set_text(cx, desktop::text(if dirty { "editor.unsaved" } else { "editor.saved" }, self.settings.language));
        self.sync_active_tab_snapshot();
        self.sync_shell_labels(cx);
    }

    pub(crate) fn mark_dirty(&mut self, cx: &mut Cx) {
        self.set_dirty(cx, true);
    }

    pub(crate) fn confirm_replace(&mut self, cx: &mut Cx, intent: ReplaceIntent) -> bool {
        if !self.dirty && !self.has_pending_work() {
            self.pending_replace = None;
            return true;
        }
        if self.pending_replace == Some(intent) {
            self.pending_replace = None;
            true
        } else {
            self.pending_replace = Some(intent);
            self.set_status(
                cx,
                "Unsaved changes — press Open again to discard, or Save first",
            );
            false
        }
    }
    pub(crate) fn new_document_tab(&mut self, cx: &mut Cx) {
        if self.has_pending_work() || self.canvas_interaction_active(cx) {
            self.set_status(
                cx,
                "Finish or cancel the current edit before creating a document",
            );
            return;
        }
        self.sync_active_tab_snapshot();
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.gesture_history_before = None;
        let number = self.tabs.len() + 1;
        let title = format!("Untitled {number}");
        let tab_id = self.allocate_tab_id();
        let mut snapshot = DocumentSnapshot::default();
        snapshot.project_path = title.clone();
        snapshot.canvas_zoom = self.settings.default_zoom;
        snapshot.dirty = false;
        self.tabs.push(DocumentTab {
            id: tab_id,
            title: title.clone(),
            path: String::new(),
            dirty: false,
            saved_snapshot: Some(snapshot.clone()),
            snapshot,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        });
        self.active_tab = self.tabs.len() - 1;
        self.apply_document_snapshot(self.tabs[self.active_tab].snapshot.clone());
        self.project_path = title;
        self.hide_annotation_popup(cx);
        self.hide_clip_popup(cx);
        self.atlases_dirty = true;
        self.set_dirty(cx, false);
        self.refresh(cx);
        self.close_menu(cx);
        self.set_status(cx, "New document tab ready");
    }

    pub(crate) fn new_project(&mut self, cx: &mut Cx) {
        self.new_document_tab(cx);
    }

    pub(crate) fn close_active_tab(&mut self, cx: &mut Cx) {
        self.request_close_tab(cx, self.active_tab);
    }

    pub(crate) fn request_close_tab(&mut self, cx: &mut Cx, index: usize) {
        if index >= self.tabs.len() {
            return;
        }
        if self.has_pending_work() || self.canvas_interaction_active(cx) {
            self.set_status(
                cx,
                "Finish or cancel the current edit before closing a document",
            );
            return;
        }
        self.sync_active_tab_snapshot();
        let tab_id = self.tabs[index].id;
        if self.tabs[index].dirty && self.settings.confirm_destructive {
            self.pending_tab_close = Some(tab_id);
            self.context_menu_open = false;
            self.menu_open = Some("close_confirm".into());
            self.overlay_pos = Some(self.shell_overlay_pos(cx, "close_confirm"));
            self.sync_shell_labels(cx);
            self.set_status(cx, "Unsaved document — choose Save, Don't Save, or Cancel");
            return;
        }
        self.close_tab_now(cx, index);
    }

    pub(crate) fn close_tab_now(&mut self, cx: &mut Cx, index: usize) {
        if index >= self.tabs.len() {
            return;
        }
        self.sync_active_tab_snapshot();
        let was_active = index == self.active_tab;
        self.tabs.remove(index);
        if self.tabs.is_empty() {
            let tab_id = self.allocate_tab_id();
            let mut snapshot = DocumentSnapshot::default();
            snapshot.project_path = "Untitled 1".into();
            self.tabs.push(DocumentTab {
                id: tab_id,
                title: "Untitled 1".into(),
                path: String::new(),
                dirty: false,
                saved_snapshot: Some(snapshot.clone()),
                snapshot,
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
            });
            self.active_tab = 0;
        } else if was_active {
            self.active_tab = index.min(self.tabs.len() - 1);
        } else if index < self.active_tab {
            self.active_tab -= 1;
        }
        let snapshot = self.tabs[self.active_tab].snapshot.clone();
        let undo_stack = self.tabs[self.active_tab].undo_stack.clone();
        let redo_stack = self.tabs[self.active_tab].redo_stack.clone();
        let preview_changed = self.apply_document_snapshot(snapshot);
        self.undo_stack = undo_stack;
        self.redo_stack = redo_stack;
        if preview_changed { self.load_preview(cx); }
        self.pending_tab_close = None;
        self.close_confirm_armed = false;
        self.refresh(cx);
        self.close_menu(cx);
        self.set_status(cx, "Document tab closed");
    }

    pub(crate) fn cancel_tab_close(&mut self, cx: &mut Cx) {
        self.pending_tab_close = None;
        self.close_menu(cx);
        self.set_status(cx, "Close cancelled");
    }

    pub(crate) fn discard_tab_close(&mut self, cx: &mut Cx) {
        let Some(tab_id) = self.pending_tab_close else {
            return;
        };
        let Some(index) = self.tabs.iter().position(|tab| tab.id == tab_id) else {
            self.cancel_tab_close(cx);
            return;
        };
        self.close_tab_now(cx, index);
    }

    pub(crate) fn save_and_close_tab(&mut self, cx: &mut Cx) {
        let Some(tab_id) = self.pending_tab_close else {
            return;
        };
        let Some(index) = self.tabs.iter().position(|tab| tab.id == tab_id) else {
            self.cancel_tab_close(cx);
            return;
        };
        if index != self.active_tab {
            self.activate_tab(cx, index);
            if index != self.active_tab {
                return;
            }
        }
        self.save_project_command(cx);
        let Some(index) = self.tabs.iter().position(|tab| tab.id == tab_id) else {
            return;
        };
        if self.tabs[index].dirty {
            self.set_status(cx, "Save did not complete; document remains open");
            return;
        }
        self.close_tab_now(cx, index);
    }

    pub(crate) fn close_other_tabs(&mut self, cx: &mut Cx) {
        self.sync_active_tab_snapshot();
        if let Some((index, _)) = self
            .tabs
            .iter()
            .enumerate()
            .find(|(index, tab)| *index != self.active_tab && tab.dirty)
        {
            // Never make a second click silently discard another document.
            // Resolve every dirty document through the same explicit dialog.
            self.request_close_tab(cx, index);
            return;
        }
        let active = self.tabs.get(self.active_tab).cloned();
        self.tabs = active.into_iter().collect();
        self.active_tab = 0;
        if let Some(active) = self.tabs.first() {
            let snapshot = active.snapshot.clone();
            let undo_stack = active.undo_stack.clone();
            let redo_stack = active.redo_stack.clone();
            self.apply_document_snapshot(snapshot);
            self.undo_stack = undo_stack;
            self.redo_stack = redo_stack;
        }
        self.refresh(cx);
        self.close_menu(cx);
        self.set_status(cx, "Other document tabs closed");
    }

    pub(crate) fn close_all_tabs(&mut self, cx: &mut Cx) {
        self.sync_active_tab_snapshot();
        if let Some((index, _)) = self.tabs.iter().enumerate().find(|(_, tab)| tab.dirty) {
            // Closing all is still a sequence of document-level decisions:
            // Save, Don't Save, or Cancel always remains explicit.
            self.request_close_tab(cx, index);
            return;
        }
        self.tabs.clear();
        self.active_tab = 0;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.gesture_history_before = None;
        self.new_document_tab(cx);
    }
}
