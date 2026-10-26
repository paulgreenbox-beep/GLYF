//! Keyboard, custom chrome, popup and pointer event routing.

use super::*;

impl App {
    pub(crate) fn route_annotation_popup_pointer(&mut self, cx: &mut Cx, event: &Event) -> bool {
        if self.pending_annotation.is_none() {
            return false;
        }
        let pointer_abs = match event {
            Event::MouseDown(pointer) => Some(pointer.abs),
            Event::TouchUpdate(update) => update
                .touches
                .iter()
                .find(|touch| matches!(touch.state, TouchState::Start))
                .map(|touch| touch.abs),
            _ => None,
        };
        let Some(pointer_abs) = pointer_abs else {
            return false;
        };
        let comment_rect = self
            .ui
            .text_input(cx, ids!(annotation_popup.comment_input))
            .area()
            .rect(cx);
        if comment_rect.contains(pointer_abs) {
            // Do not forward the pointer through the modal root: native focus
            // is sufficient here and avoids a stale finger capture on Windows.
            self.ui
                .text_input(cx, ids!(annotation_popup.comment_input))
                .set_key_focus(cx);
            return true;
        }
        let hits_button = [
            ids!(annotation_popup.actions.cancel),
            ids!(annotation_popup.actions.commit),
        ]
        .into_iter()
        .any(|id| self.ui.button(cx, id).area().rect(cx).contains(pointer_abs));
        if hits_button {
            self.ui
                .widget(cx, ids!(annotation_popup))
                .handle_event(cx, event, &mut Scope::empty());
        }
        // Every other pointer event is click-away inside the modal boundary:
        // it is intentionally ignored and never reaches Canvas.
        true
    }

    pub(crate) fn text_input_has_focus(&self, cx: &Cx) -> bool {
        if self.has_pending_work() {
            return true;
        }
        [
            ids!(image_path_input),
            ids!(name_input),
            ids!(comment_input),
            ids!(part_name_input),
            ids!(part_source_input),
            ids!(part_parent_input),
            ids!(clip_x_input),
            ids!(clip_y_input),
            ids!(clip_w_input),
            ids!(clip_h_input),
            ids!(rel1_x_input),
            ids!(rel1_y_input),
            ids!(rel2_x_input),
            ids!(rel2_y_input),
            ids!(part_comment_input),
        ]
        .into_iter()
        .any(|id| cx.has_key_focus(self.ui.widget(cx, id).area()))
    }

    pub(crate) fn cancel_active_canvas_interaction(&mut self, cx: &mut Cx) -> bool {
        let cancelled = if self.source_mode {
            self.ui
                .widget(cx, ids!(source_canvas))
                .borrow_mut::<SourceCanvas>()
                .map(|mut canvas| canvas.cancel_interaction(cx))
                .unwrap_or(false)
        } else {
            self.ui
                .widget(cx, ids!(canvas_overlay))
                .borrow_mut::<SpatialCanvas>()
                .map(|mut canvas| canvas.cancel_interaction(cx))
                .unwrap_or(false)
        };
        if cancelled {
            self.gesture_history_before = None;
            self.set_status(cx, "Interaction cancelled");
        }
        cancelled
    }

    pub(crate) fn canvas_interaction_active(&self, cx: &Cx) -> bool {
        if self.source_mode {
            self.ui
                .widget(cx, ids!(source_canvas))
                .borrow::<SourceCanvas>()
                .map(|canvas| canvas.interaction_active())
                .unwrap_or(false)
        } else {
            self.ui
                .widget(cx, ids!(canvas_overlay))
                .borrow::<SpatialCanvas>()
                .map(|canvas| canvas.interaction_active())
                .unwrap_or(false)
        }
    }
    pub(crate) fn handle_close_request(&mut self, cx: &mut Cx, event: &Event) {
        let Event::WindowCloseRequested(request) = event else {
            return;
        };
        if (self.dirty || self.has_pending_work()) && !self.close_confirm_armed {
            request.accept_close.set(false);
            self.close_confirm_armed = true;
            self.set_status(
                cx,
                "Unsaved changes — close again to discard, or Save the document",
            );
        }
    }

    pub(crate) fn request_editor_exit(&mut self, cx: &mut Cx) {
        if self.dirty || self.has_pending_work() {
            self.set_status(cx, "Unsaved changes — save before Exit");
        } else {
            cx.quit();
        }
    }

    pub(crate) fn update_toolbar_tooltip(&mut self, cx: &mut Cx, event: &Event) {
        let Event::MouseMove(pointer) = event else {
            if matches!(event, Event::MouseLeave(_) | Event::MouseDown(_)) {
                if self.hovered_toolbar_tooltip.take().is_some() {
                    self.ui.tooltip(cx, ids!(toolbar_tooltip)).hide(cx);
                }
            }
            return;
        };
        type Hint = (&'static [LiveId], &'static str, &'static str, &'static str);
        let common: &[Hint] = &[
            (ids!(new_project), "new", "Создать документ · Ctrl+N", "New document · Ctrl+N"),
            (ids!(save_project), "save", "Сохранить документ · Ctrl+S", "Save document · Ctrl+S"),
            (ids!(sidebar_images), "images", "Показать изображения", "Show images"),
            (ids!(sidebar_layers), "layers", "Показать слои", "Show layers"),
        ];
        let spatial: &[Hint] = &[
            (ids!(tool_select), "select", "Выбрать объект", "Select object"),
            (ids!(tool_point), "point", "Поставить точку", "Add point"),
            (ids!(tool_region), "region", "Выделить область", "Select region"),
            (ids!(canvas_fit), "fit", "Вписать в окно", "Fit to window"),
            (ids!(canvas_100), "actual", "Масштаб 100%", "Actual size"),
            (ids!(canvas_zoom_out), "zoom_out", "Уменьшить масштаб", "Zoom out"),
            (ids!(canvas_zoom_in), "zoom_in", "Увеличить масштаб", "Zoom in"),
        ];
        let source: &[Hint] = &[
            (ids!(source_back), "back", "Вернуться к композиции", "Back to composition"),
            (ids!(source_select), "source_select", "Выбрать на изображении", "Select in image"),
            (ids!(source_clip), "clip", "Вырезать область", "Clip region"),
            (ids!(source_fit), "source_fit", "Вписать изображение", "Fit image"),
            (ids!(source_100), "source_actual", "Масштаб 100%", "Actual size"),
            (ids!(source_zoom_out), "source_zoom_out", "Уменьшить масштаб", "Zoom out"),
            (ids!(source_zoom_in), "source_zoom_in", "Увеличить масштаб", "Zoom in"),
        ];
        let active = if self.source_mode { source } else { spatial };
        let hovered = if self.menu_open.is_some() || self.context_menu_open {
            None
        } else {
            common.iter().chain(active.iter()).find_map(|(id, key, ru, en)| {
                let rect = self.ui.button(cx, id).area().rect(cx);
                if rect.size.x > 0.0 && rect.size.y > 0.0 && rect.contains(pointer.abs) {
                    Some((*key, if self.settings.language == Language::Russian { *ru } else { *en }, rect))
                } else {
                    None
                }
            })
        };
        if let Some((key, label, rect)) = hovered {
            if self.hovered_toolbar_tooltip.as_deref() != Some(key) {
                self.hovered_toolbar_tooltip = Some(key.to_owned());
                let tooltip = self.ui.tooltip(cx, ids!(toolbar_tooltip));
                tooltip.show_with_options(
                    cx,
                    dvec2(rect.pos.x, rect.pos.y + rect.size.y + 8.0),
                    label,
                );
            }
        } else if self.hovered_toolbar_tooltip.take().is_some() {
            self.ui.tooltip(cx, ids!(toolbar_tooltip)).hide(cx);
        }
    }

    /// Owns physical pointer routing for the in-client chrome. This avoids the
    /// native caption hit-test path while retaining Makepad widgets purely as
    /// the visual surface and accessibility anchor.
    pub(crate) fn handle_custom_chrome_pointer(&mut self, cx: &mut Cx, event: &Event) -> bool {
        if let Event::MouseDown(pointer) = event {
            let abs = pointer.abs;
            if self
                .ui
                .button(cx, ids!(tab_new))
                .area()
                .rect(cx)
                .contains(abs)
            {
                self.new_document_tab(cx);
                return true;
            }
            if self
                .ui
                .button(cx, ids!(tab_overflow))
                .area()
                .rect(cx)
                .contains(abs)
            {
                self.open_tab_overflow(cx);
                return true;
            }
            if self
                .ui
                .button(cx, ids!(chrome_minimize))
                .area()
                .rect(cx)
                .contains(abs)
            {
                #[cfg(windows)]
                minimize_editor_window();
                return true;
            }
            if self
                .ui
                .button(cx, ids!(chrome_maximize))
                .area()
                .rect(cx)
                .contains(abs)
            {
                #[cfg(windows)]
                toggle_editor_window_maximized();
                return true;
            }
            if self
                .ui
                .button(cx, ids!(chrome_close))
                .area()
                .rect(cx)
                .contains(abs)
            {
                self.request_editor_exit(cx);
                return true;
            }
            if let Some(index) = self.tabs.iter().enumerate().find_map(|(index, tab)| {
                self.document_tab_close_rect(cx, tab)
                    .filter(|rect| rect.contains(abs))
                    .map(|_| index)
            }) {
                self.request_close_tab(cx, index);
                return true;
            }
            if let Some(index) = self.tabs.iter().enumerate().find_map(|(index, tab)| {
                self.document_tab_rect(cx, tab)
                    .filter(|rect| rect.contains(abs))
                    .map(|_| index)
            }) {
                self.activate_tab(cx, index);
                return true;
            }
            // Tabs and window controls have already claimed their hit areas.
            // The rest of the title row (including unused tab-strip space) is
            // a drag surface, just like the blank area of a browser tab bar.
            let drag_rect = self.ui.view(cx, ids!(chrome_bar)).area().rect(cx);
            if drag_rect.size.x > 0.0 && drag_rect.size.y > 0.0 && drag_rect.contains(abs) {
                #[cfg(windows)]
                begin_editor_window_drag();
                return true;
            }
        }
        false
    }

    /// Opens the Canvas context menu from a position already expressed in window
    /// coordinates. Overlay visibility and placement stay owned by App.
    pub(crate) fn open_canvas_context_menu(&mut self, cx: &mut Cx, pos: DVec2) {
        self.sync_menu_enabled(cx);
        self.context_menu_open = true;
        self.menu_open = None;
        self.menu_chain.close();
        let viewport = self.ui.view(cx, ids!(main_layout)).area().rect(cx);
        let context_size = self.shell_overlay_size("context");
        let min_x = viewport.pos.x + 8.0;
        let min_y = viewport.pos.y + 8.0;
        self.overlay_pos = Some(dvec2(
            (pos.x + 4.0).clamp(
                min_x,
                (viewport.pos.x + viewport.size.x - context_size.x - 8.0).max(min_x),
            ),
            (pos.y + 4.0).clamp(
                min_y,
                (viewport.pos.y + viewport.size.y - context_size.y - 8.0).max(min_y),
            ),
        ));
        self.sync_shell_labels(cx);
        self.set_status(
            cx,
            format!("Context menu opened at {:.0}, {:.0}", pos.x, pos.y),
        );
    }

    /// Raw secondary-click routing is deliberately above widget hit-testing:
    /// a Canvas background can be covered by nested views, but the root app is
    /// the single owner of the shell overlay and always receives MouseDown.
    pub(crate) fn handle_canvas_context_pointer(&mut self, cx: &mut Cx, event: &Event) -> bool {
        let Event::MouseDown(pointer) = event else {
            return false;
        };
        if !pointer.button.contains(MouseButton::SECONDARY)
            || self.source_mode
            || self.has_pending_work()
            || !self.canvas_rect(cx).contains(pointer.abs)
        {
            return false;
        }
        self.open_canvas_context_menu(cx, pointer.abs);
        true
    }

    pub(crate) fn handle_global_key_event(&mut self, cx: &mut Cx, event: &Event) {
        let Event::KeyDown(key) = event else { return };
        if key.modifiers.alt && key.key_code == KeyCode::F4 {
            if !self.dirty {
                cx.quit();
            } else {
                self.set_status(cx, "Unsaved changes — save before Exit");
            }
            return;
        }
        // Escape always dismisses transient UI, even when a text field still
        // has focus. This keeps menus and dialogs recoverable by keyboard.
        if key.key_code == KeyCode::Escape {
            if self.pending_part_popup.is_some() {
                self.cancel_part_popup(cx);
            } else if self.pending_clip.is_some() {
                self.cancel_clip(cx);
            } else if self.pending_annotation.is_some() {
                self.cancel_annotation(cx);
            } else if self.menu_open.as_deref() == Some("close_confirm") {
                self.cancel_tab_close(cx);
            } else if self.menu_open.is_some() || self.context_menu_open {
                self.close_menu(cx);
            } else if self.cancel_active_canvas_interaction(cx) {
                // The canvas restored the gesture origin.
            } else if self.source_mode && self.source_tool != SourceTool::Select {
                self.set_source_tool(cx, SourceTool::Select);
            } else if self.source_mode {
                self.leave_source_mode(cx);
            } else if self.tool != CanvasTool::Select {
                self.set_canvas_tool(cx, CanvasTool::Select);
            }
            return;
        }
        // Native text controls own their editing shortcuts. In particular,
        // Ctrl+A must select text in the active field rather than select
        // Canvas content while an annotation popup is open.
        if self.text_input_has_focus(cx) {
            return;
        }
        let command = key.modifiers.control || key.modifiers.logo;
        if command {
            match key.key_code {
                KeyCode::KeyS if key.modifiers.shift => self.save_project_as(cx),
                KeyCode::KeyS => self.save_project_command(cx),
                KeyCode::KeyO => self.open_project_action(cx),
                KeyCode::KeyN => self.new_document_tab(cx),
                KeyCode::KeyW => self.close_active_tab(cx),
                KeyCode::KeyZ if key.modifiers.shift => self.redo(cx),
                KeyCode::KeyZ => self.undo(cx),
                KeyCode::KeyY => self.redo(cx),
                KeyCode::KeyX => self.cut_selection(cx),
                KeyCode::KeyC => self.copy_selection(cx),
                KeyCode::KeyV => self.paste_selection(cx),
                KeyCode::KeyD => self.duplicate_selection(cx),
                KeyCode::KeyA => self.select_all(cx),
                KeyCode::Tab if key.modifiers.shift => self.previous_tab(cx),
                KeyCode::Tab => self.next_tab(cx),
                KeyCode::Equals | KeyCode::NumpadAdd => self.adjust_canvas_zoom(cx, 1.25),
                KeyCode::Minus | KeyCode::NumpadSubtract => self.adjust_canvas_zoom(cx, 0.8),
                KeyCode::Key0 => self.fit_canvas(cx),
                _ => {}
            }
            return;
        }
        match key.key_code {
            KeyCode::Delete if !self.text_input_has_focus(cx) => {
                if self.source_mode && self.pending_clip.is_some() {
                    self.cancel_clip(cx);
                } else if self.selected_annotation.is_some() {
                    self.delete_annotation(cx);
                } else if self.selected_part.is_some() {
                    self.remove_part(cx);
                }
            }
            KeyCode::KeyV if !self.text_input_has_focus(cx) => {
                if self.source_mode {
                    self.set_source_tool(cx, SourceTool::Select);
                } else {
                    self.set_canvas_tool(cx, CanvasTool::Select);
                }
            }
            KeyCode::KeyP if !self.text_input_has_focus(cx) => {
                if self.source_mode {
                    self.leave_source_mode(cx);
                }
                self.set_canvas_tool(cx, CanvasTool::Point);
            }
            KeyCode::KeyR if !self.text_input_has_focus(cx) => {
                if self.source_mode {
                    self.leave_source_mode(cx);
                }
                self.set_canvas_tool(cx, CanvasTool::Region);
            }
            KeyCode::KeyC if !self.text_input_has_focus(cx) => {
                if !self.source_mode {
                    self.enter_source_mode(cx);
                }
                if self.source_mode {
                    self.set_source_tool(cx, SourceTool::Clip);
                }
            }
            _ => {}
        }
    }
}
