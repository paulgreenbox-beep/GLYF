//! Semantic UI commands. Existing IDs and action dispatch are preserved.

use super::*;

impl App {
    #[cfg(windows)]
    pub(crate) fn handle_accessibility_action(&mut self, cx: &mut Cx, request: AccessibilityActionRequest) {
        if let AccessibilityAction::SetValue(value) = &request.action {
            if request.node_id.starts_with("settings.") {
                let _ = self.set_settings_semantic_value(cx, &request.node_id, value);
                self.refresh_accessibility(cx);
                return;
            }
        }
        if let Some(raw) = request
            .node_id
            .strip_prefix("image.item.")
            .or_else(|| request.node_id.strip_prefix("image.canvas."))
        {
            if let Ok(index) = raw.parse::<usize>() {
                if matches!(request.action, AccessibilityAction::Invoke)
                    && index < self.images.len()
                {
                    self.selected = Some(index);
                    self.load_preview(cx);
                    self.refresh(cx);
                    self.set_status(cx, "Image selected");
                }
            }
            self.refresh_accessibility(cx);
            return;
        }
        if let Some(raw) = request
            .node_id
            .strip_prefix("part.item.")
            .or_else(|| request.node_id.strip_prefix("part.canvas."))
        {
            if let Ok(index) = raw.parse::<usize>() {
                if matches!(request.action, AccessibilityAction::Invoke)
                    && index < self.current_parts().len()
                {
                    self.selected_part = Some(index);
                    self.selected_annotation = None;
                    self.sync_spatial(cx);

                    self.set_status(cx, "Part selected");
                }
            }
            self.refresh_accessibility(cx);
            return;
        }
        if let Some(raw) = request
            .node_id
            .strip_prefix("annotation.item.")
            .or_else(|| request.node_id.strip_prefix("annotation.canvas."))
        {
            if let Ok(index) = raw.parse::<usize>() {
                if matches!(request.action, AccessibilityAction::Invoke)
                    && index < self.annotations.len()
                {
                    self.selected_annotation = Some(index);
                    self.selected_part = None;
                    self.sync_spatial(cx);

                    self.set_status(cx, "Annotation selected");
                }
            }
            self.refresh_accessibility(cx);
            return;
        }
        if let Some(raw) = request
            .node_id
            .strip_prefix("tab.item.")
            .and_then(|raw| raw.strip_suffix(".close"))
        {
            if let Ok(index) = raw.parse::<usize>() {
                if matches!(request.action, AccessibilityAction::Invoke) {
                    self.request_close_tab(cx, index);
                }
            }
            self.refresh_accessibility(cx);
            return;
        }
        if let Some(raw) = request.node_id.strip_prefix("tab.item.") {
            if let Ok(index) = raw.parse::<usize>() {
                if matches!(request.action, AccessibilityAction::Invoke) {
                    self.activate_tab(cx, index);
                }
            }
            self.refresh_accessibility(cx);
            return;
        }
        if let Some(raw) = request.node_id.strip_prefix("recent.item.") {
            if let Ok(index) = raw.parse::<usize>() {
                if matches!(request.action, AccessibilityAction::Invoke) {
                    if let Some(path) = desktop::recent_menu_entries(&self.settings.recent_files)
                        .get(index)
                        .map(|entry| entry.path.clone())
                    {
                        self.open_recent_path(cx, &path);
                    }
                }
            }
            self.refresh_accessibility(cx);
            return;
        }
        if request.node_id == "recent.clear" {
            if matches!(request.action, AccessibilityAction::Invoke) {
                self.clear_recent(cx);
            }
            self.refresh_accessibility(cx);
            return;
        }
        let close_menu_after_action = matches!(
            request.node_id.as_str(),
            "menu.file.new"
                | "menu.file.open"
                | "menu.file.recent"
                | "menu.file.save"
                | "menu.file.save_as"
                | "menu.file.save_pack"
                | "menu.file.exit"
                | "menu.edit.undo"
                | "menu.edit.redo"
                | "menu.edit.cut"
                | "menu.edit.copy"
                | "menu.edit.paste"
                | "menu.edit.duplicate"
                | "menu.edit.delete"
                | "menu.edit.select_all"
                | "menu.view.zoom_in"
                | "menu.view.zoom_out"
                | "menu.view.fit"
                | "menu.window.next"
                | "menu.window.previous"
                | "menu.window.close_others"
                | "menu.window.close_all"
                | "menu.language.english"
                | "menu.language.russian"
        );
        match (request.node_id.as_str(), request.action) {
            ("project.path", AccessibilityAction::SetValue(value)) => self.project_path = value,
            ("image.path", AccessibilityAction::SetValue(value)) => self
                .ui
                .text_input(cx, ids!(image_path_input))
                .set_text(cx, &value),
            ("image.name", AccessibilityAction::SetValue(value)) => self
                .ui
                .text_input(cx, ids!(name_input))
                .set_text(cx, &value),
            ("annotation.role", AccessibilityAction::SetValue(value)) => {
                let role = value.trim().to_owned();
                if !role.is_empty() {
                    let before = self.capture_document_snapshot();
                    let mut changed = false;
                    if let Some(index) = self.selected_annotation {
                        if let Some(annotation) = self.annotations.get_mut(index) {
                            if annotation.role != role {
                                annotation.role = role;
                                changed = true;
                            }
                        }
                    }
                    if changed {
                        self.mark_dirty(cx);
                        self.sync_spatial(cx);
                        self.record_history("Update annotation role", before);
                    }
                }
            }
            ("image.comment", AccessibilityAction::SetValue(value)) => self
                .ui
                .text_input(cx, ids!(comment_input))
                .set_text(cx, &value),
            ("menu.file", AccessibilityAction::Invoke) => self.toggle_menu(cx, "file"),
            ("menu.edit", AccessibilityAction::Invoke) => self.toggle_menu(cx, "edit"),
            ("menu.view", AccessibilityAction::Invoke) => self.toggle_menu(cx, "view"),
            ("menu.window", AccessibilityAction::Invoke) => self.toggle_menu(cx, "window"),
            ("menu.language", AccessibilityAction::Invoke) => self.toggle_menu(cx, "language"),
            ("menu.help", AccessibilityAction::Invoke) => self.toggle_menu(cx, "help"),
            ("tab.new", AccessibilityAction::Invoke) => self.new_document_tab(cx),
            ("tab.overflow", AccessibilityAction::Invoke) => self.open_tab_overflow(cx),
            ("menu.file.new", AccessibilityAction::Invoke) => self.new_document_tab(cx),
            ("menu.file.open" | "dialog.open", AccessibilityAction::Invoke) => {
                self.open_project_action(cx)
            }
            ("menu.file.open_recent" | "menu.file.recent", AccessibilityAction::Invoke) => {
                self.open_recent_submenu(cx)
            }
            ("menu.file.save", AccessibilityAction::Invoke) => self.save_project_command(cx),
            ("menu.file.save_as" | "dialog.save_as", AccessibilityAction::Invoke) => {
                self.save_project_as(cx)
            }
            ("menu.file.save_pack", AccessibilityAction::Invoke) => self.save_project(cx),
            ("menu.file.close", AccessibilityAction::Invoke) => self.close_active_tab(cx),
            ("menu.file.exit", AccessibilityAction::Invoke) => {
                if self.dirty {
                    self.set_status(cx, "Unsaved changes — save before Exit");
                } else {
                    cx.quit();
                }
            }
            ("history.undo" | "menu.edit.undo", AccessibilityAction::Invoke) => self.undo(cx),
            ("context.copy", AccessibilityAction::Invoke) => {
                self.copy_selection(cx);
                self.close_menu(cx);
            }
            ("context.cut", AccessibilityAction::Invoke) => {
                self.cut_selection(cx);
                self.close_menu(cx);
            }
            ("context.paste", AccessibilityAction::Invoke) => {
                self.paste_selection(cx);
                self.close_menu(cx);
            }
            ("context.duplicate", AccessibilityAction::Invoke) => {
                self.duplicate_selection(cx);
                self.close_menu(cx);
            }
            ("context.delete", AccessibilityAction::Invoke) => {
                if self.selected_annotation.is_some() {
                    self.delete_annotation(cx);
                } else {
                    self.remove_part(cx);
                }
                self.close_menu(cx);
            }
            ("history.redo" | "menu.edit.redo", AccessibilityAction::Invoke) => self.redo(cx),
            ("menu.edit.cut" | "clipboard.cut", AccessibilityAction::Invoke) => {
                self.cut_selection(cx)
            }
            ("menu.edit.copy" | "clipboard.copy", AccessibilityAction::Invoke) => {
                self.copy_selection(cx);
                self.close_menu(cx);
            }
            ("menu.edit.paste" | "clipboard.paste", AccessibilityAction::Invoke) => {
                self.paste_selection(cx)
            }
            ("menu.edit.duplicate" | "clipboard.duplicate", AccessibilityAction::Invoke) => {
                self.duplicate_selection(cx)
            }
            ("menu.edit.delete", AccessibilityAction::Invoke) => {
                if self.selected_annotation.is_some() {
                    self.delete_annotation(cx);
                } else {
                    self.remove_part(cx);
                }
            }
            ("menu.edit.select_all", AccessibilityAction::Invoke) => self.select_all(cx),
            ("menu.settings", AccessibilityAction::Invoke) => self.open_shell_panel(cx, "settings"),
            ("settings.restore", AccessibilityAction::Invoke) => self.toggle_restore_session(cx),
            ("settings.category.general", AccessibilityAction::Invoke) => self.set_settings_category(cx, SettingsCategory::General),
            ("settings.category.appearance", AccessibilityAction::Invoke) => self.set_settings_category(cx, SettingsCategory::Appearance),
            ("settings.category.editor", AccessibilityAction::Invoke) => self.set_settings_category(cx, SettingsCategory::Editor),
            ("settings.category.files", AccessibilityAction::Invoke) => self.set_settings_category(cx, SettingsCategory::Files),
            ("settings.category.accessibility", AccessibilityAction::Invoke) => self.set_settings_category(cx, SettingsCategory::Accessibility),
            ("settings.general.restore" | "settings.files.reopen_last", AccessibilityAction::Invoke) => self.toggle_restore_session(cx),
            ("settings.editor.resources", AccessibilityAction::Invoke) => self.toggle_sidebar(cx, "resources"),
            ("settings.editor.layers", AccessibilityAction::Invoke) => self.toggle_sidebar(cx, "layers"),
            ("settings.editor.status_bar", AccessibilityAction::Invoke) => {
                self.settings.panel_status_bar = !self.settings.panel_status_bar;
                self.commit_desktop_settings(cx, true);
            }
            ("settings.editor.confirm_destructive", AccessibilityAction::Invoke) => {
                self.settings.confirm_destructive = !self.settings.confirm_destructive;
                self.commit_desktop_settings(cx, false);
            }
            ("settings.accessibility.reduced_motion", AccessibilityAction::Invoke) => {
                self.style_profile.motion = if matches!(self.style_profile.motion, MotionStyle::None | MotionStyle::Reduced) { MotionStyle::Subtle } else { MotionStyle::Reduced };
                self.commit_desktop_settings(cx, true);
            }
            ("settings.accessibility.large_font", AccessibilityAction::Invoke) => {
                self.settings.font_size = if self.settings.font_size == FontSizeChoice::Large { FontSizeChoice::Normal } else { FontSizeChoice::Large };
                self.commit_desktop_settings(cx, true);
            }
            ("settings.reset_appearance", AccessibilityAction::Invoke) => {
                let defaults = DesktopSettings::default();
                self.style_profile = defaults.style_profile;
                self.settings.font_size = defaults.font_size;
                self.settings.ui_scale = defaults.ui_scale;
                self.settings.density = defaults.density;
                self.commit_desktop_settings(cx, true);
            }
            ("settings.reset_all", AccessibilityAction::Invoke) => self.ui.widget(cx, ids!(settings_reset_confirm)).set_visible(cx, true),
            ("settings.reset.cancel", AccessibilityAction::Invoke) => self.ui.widget(cx, ids!(settings_reset_confirm)).set_visible(cx, false),
            ("settings.reset.yes", AccessibilityAction::Invoke) => {
                self.settings = DesktopSettings::default();
                self.style_profile = self.settings.style_profile;
                self.panel_resources = self.settings.panel_resources;
                self.panel_layers = self.settings.panel_layers;
                self.ui.widget(cx, ids!(settings_reset_confirm)).set_visible(cx, false);
                self.commit_desktop_settings(cx, true);
            }
            ("settings.close" | "about.close" | "shortcuts.close", AccessibilityAction::Invoke) => {
                self.close_menu(cx)
            }
            ("menu.view.zoom_in", AccessibilityAction::Invoke) => self.adjust_canvas_zoom(cx, 1.25),
            ("menu.view.zoom_out", AccessibilityAction::Invoke) => self.adjust_canvas_zoom(cx, 0.8),
            ("menu.view.fit", AccessibilityAction::Invoke) => self.fit_canvas(cx),
            ("menu.view.resources" | "sidebar.images", AccessibilityAction::Invoke) => {
                self.toggle_sidebar(cx, "resources");
                self.close_menu(cx);
            }
            ("menu.view.layers" | "sidebar.layers", AccessibilityAction::Invoke) => {
                self.toggle_sidebar(cx, "layers");
                self.close_menu(cx);
            }
            ("menu.window.next", AccessibilityAction::Invoke) => self.next_tab(cx),
            ("menu.window.previous", AccessibilityAction::Invoke) => self.previous_tab(cx),
            ("menu.window.close", AccessibilityAction::Invoke) => self.close_active_tab(cx),
            ("menu.window.close_others", AccessibilityAction::Invoke) => self.close_other_tabs(cx),
            ("menu.window.close_all", AccessibilityAction::Invoke) => self.close_all_tabs(cx),
            ("menu.language.english", AccessibilityAction::Invoke) => {
                self.set_language(cx, Language::English)
            }
            ("menu.language.russian", AccessibilityAction::Invoke) => {
                self.set_language(cx, Language::Russian)
            }
            ("menu.help.about", AccessibilityAction::Invoke) => self.open_shell_panel(cx, "about"),
            ("menu.help.shortcuts", AccessibilityAction::Invoke) => {
                self.open_shell_panel(cx, "shortcuts")
            }
            ("window.minimize", AccessibilityAction::Invoke) => minimize_editor_window(),
            ("window.maximize", AccessibilityAction::Invoke) => toggle_editor_window_maximized(),
            ("window.close", AccessibilityAction::Invoke) => self.request_editor_exit(cx),
            ("tab.close.confirm.cancel", AccessibilityAction::Invoke) => self.cancel_tab_close(cx),
            ("tab.close.confirm.dont_save", AccessibilityAction::Invoke) => {
                self.discard_tab_close(cx)
            }
            ("tab.close.confirm.save", AccessibilityAction::Invoke) => self.save_and_close_tab(cx),
            ("project.new", AccessibilityAction::Invoke) => self.new_project(cx),
            ("project.open", AccessibilityAction::Invoke) => self.open_project_action(cx),
            ("project.save", AccessibilityAction::Invoke) => self.save_project_command(cx),
            ("image.add", AccessibilityAction::Invoke) => self.add_image_action(cx),
            ("image.remove", AccessibilityAction::Invoke) => self.remove_image(cx),
            ("image.update", AccessibilityAction::Invoke) => self.update_image(cx),
            ("source.mode", AccessibilityAction::Invoke) => self.enter_source_mode(cx),
            ("source.back", AccessibilityAction::Invoke) => self.leave_source_mode(cx),
            ("tool.source.select", AccessibilityAction::Invoke) => {
                self.set_source_tool(cx, SourceTool::Select)
            }
            ("tool.source.clip", AccessibilityAction::Invoke) => {
                self.set_source_tool(cx, SourceTool::Clip)
            }
            ("source.fit", AccessibilityAction::Invoke) => self.fit_source(cx),
            ("source.100", AccessibilityAction::Invoke) => self.source_100(cx),
            ("source.zoom.out", AccessibilityAction::Invoke) => self.adjust_source_zoom(cx, 0.8),
            ("source.zoom.in", AccessibilityAction::Invoke) => self.adjust_source_zoom(cx, 1.25),
            ("canvas.fit", AccessibilityAction::Invoke) => self.fit_canvas(cx),
            ("canvas.100", AccessibilityAction::Invoke) => self.canvas_100(cx),
            ("canvas.zoom.out", AccessibilityAction::Invoke) => self.adjust_canvas_zoom(cx, 0.8),
            ("canvas.zoom.in", AccessibilityAction::Invoke) => self.adjust_canvas_zoom(cx, 1.25),
            ("image.path", AccessibilityAction::Focus) => self
                .ui
                .text_input(cx, ids!(image_path_input))
                .set_key_focus(cx),
            ("image.name", AccessibilityAction::Focus) => {
                self.ui.text_input(cx, ids!(name_input)).set_key_focus(cx)
            }
            ("image.comment", AccessibilityAction::Focus) => self
                .ui
                .text_input(cx, ids!(comment_input))
                .set_key_focus(cx),

            ("part.name", AccessibilityAction::SetValue(value)) => self
                .ui
                .text_input(cx, ids!(part_name_input))
                .set_text(cx, &value),
            ("part.source", AccessibilityAction::SetValue(value)) => self
                .ui
                .text_input(cx, ids!(part_source_input))
                .set_text(cx, &value),
            ("part.parent", AccessibilityAction::SetValue(value)) => self
                .ui
                .text_input(cx, ids!(part_parent_input))
                .set_text(cx, &value),
            ("part.clip.x", AccessibilityAction::SetValue(value)) => self
                .ui
                .text_input(cx, ids!(clip_x_input))
                .set_text(cx, &value),
            ("part.clip.y", AccessibilityAction::SetValue(value)) => self
                .ui
                .text_input(cx, ids!(clip_y_input))
                .set_text(cx, &value),
            ("part.clip.w", AccessibilityAction::SetValue(value)) => self
                .ui
                .text_input(cx, ids!(clip_w_input))
                .set_text(cx, &value),
            ("part.clip.h", AccessibilityAction::SetValue(value)) => self
                .ui
                .text_input(cx, ids!(clip_h_input))
                .set_text(cx, &value),
            ("part.rel1.x", AccessibilityAction::SetValue(value)) => self
                .ui
                .text_input(cx, ids!(rel1_x_input))
                .set_text(cx, &value),
            ("part.rel1.y", AccessibilityAction::SetValue(value)) => self
                .ui
                .text_input(cx, ids!(rel1_y_input))
                .set_text(cx, &value),
            ("part.rel2.x", AccessibilityAction::SetValue(value)) => self
                .ui
                .text_input(cx, ids!(rel2_x_input))
                .set_text(cx, &value),
            ("part.rel2.y", AccessibilityAction::SetValue(value)) => self
                .ui
                .text_input(cx, ids!(rel2_y_input))
                .set_text(cx, &value),
            ("part.comment", AccessibilityAction::SetValue(value)) => self
                .ui
                .text_input(cx, ids!(part_comment_input))
                .set_text(cx, &value),
            ("part.apply", AccessibilityAction::Invoke) => self.apply_part(cx),
            ("part.edit_clip", AccessibilityAction::Invoke) => {
                if let Some(index) = self.selected_part {
                    self.begin_edit_clip(cx, index);
                }
            }
            ("tool.select", AccessibilityAction::Invoke) => {
                self.set_canvas_tool(cx, CanvasTool::Select)
            }
            ("tool.point", AccessibilityAction::Invoke) => {
                self.set_canvas_tool(cx, CanvasTool::Point)
            }
            ("tool.region", AccessibilityAction::Invoke) => {
                self.set_canvas_tool(cx, CanvasTool::Region)
            }
            ("annotation.region.input", AccessibilityAction::SetValue(value)) => {
                self.set_region_from_accessibility(cx, &value)
            }
            ("annotation.edit", AccessibilityAction::Invoke) => {
                if let Some(index) = self.selected_annotation {
                    self.begin_edit_annotation(cx, index);
                }
            }
            ("annotation.delete", AccessibilityAction::Invoke) => self.delete_annotation(cx),
            ("annotation.popup.commit" | "annotation.popup.done", AccessibilityAction::Invoke) => {
                self.commit_annotation(cx)
            }
            ("annotation.popup.cancel", AccessibilityAction::Invoke) => self.cancel_annotation(cx),
            ("clip.popup.name" | "part.popup.name", AccessibilityAction::SetValue(value)) => {
                if let Some(pending) = self.pending_part_popup.as_mut() {
                    pending.draft_name = value.clone();
                } else if let Some(pending) = self.pending_clip.as_mut() {
                    pending.draft_name = value.clone();
                }
                if let Some(mut popup) = self
                    .ui
                    .widget(cx, ids!(clip_popup))
                    .borrow_mut::<ClipPopup>()
                {
                    popup.set_name(cx, &value);
                }
            }
            ("clip.popup.comment" | "part.popup.comment", AccessibilityAction::SetValue(value)) => {
                if let Some(pending) = self.pending_part_popup.as_mut() {
                    pending.draft_comment = value.clone();
                } else if let Some(pending) = self.pending_clip.as_mut() {
                    pending.draft_comment = value.clone();
                }
                if let Some(mut popup) = self
                    .ui
                    .widget(cx, ids!(clip_popup))
                    .borrow_mut::<ClipPopup>()
                {
                    popup.set_comment(cx, &value);
                }
            }
            ("clip.input", AccessibilityAction::SetValue(value)) => {
                self.set_source_clip_from_accessibility(cx, &value)
            }
            ("clip.popup.create" | "part.popup.commit", AccessibilityAction::Invoke) => {
                if self.pending_part_popup.is_some() {
                    self.commit_part_popup(cx);
                } else {
                    self.commit_clip(cx);
                }
            }
            ("clip.popup.cancel" | "part.popup.cancel", AccessibilityAction::Invoke) => {
                if self.pending_part_popup.is_some() {
                    self.cancel_part_popup(cx);
                } else {
                    self.cancel_clip(cx);
                }
            }
            ("annotation.popup.comment", AccessibilityAction::SetValue(value)) => {
                self.annotation_draft = value.clone();
                self.ui
                    .text_input(cx, ids!(annotation_popup.comment_input))
                    .set_text(cx, &value);
            }
            ("clip.popup.name" | "part.popup.name", AccessibilityAction::Focus) => {
                if let Some(mut popup) = self
                    .ui
                    .widget(cx, ids!(clip_popup))
                    .borrow_mut::<ClipPopup>()
                {
                    popup.focus_name(cx);
                }
            }
            ("clip.popup.comment" | "part.popup.comment", AccessibilityAction::Focus) => {
                if let Some(mut popup) = self
                    .ui
                    .widget(cx, ids!(clip_popup))
                    .borrow_mut::<ClipPopup>()
                {
                    popup.focus_comment(cx);
                }
            }
            ("annotation.popup.comment", AccessibilityAction::Focus) => {
                self.ui
                    .text_input(cx, ids!(annotation_popup.comment_input))
                    .set_key_focus(cx);
            }
            ("part.add", AccessibilityAction::Invoke) => self.add_part(cx, false),
            ("part.add_clip", AccessibilityAction::Invoke) => self.add_part(cx, true),
            ("part.remove", AccessibilityAction::Invoke) => self.remove_part(cx),
            ("part.up", AccessibilityAction::Invoke) => self.move_part(cx, -1),
            ("part.down", AccessibilityAction::Invoke) => self.move_part(cx, 1),
            ("part.front", AccessibilityAction::Invoke) => self.move_part_to(cx, true),
            ("part.back", AccessibilityAction::Invoke) => self.move_part_to(cx, false),
            _ => return,
        }
        if close_menu_after_action {
            self.close_menu(cx);
        }
        self.refresh_accessibility(cx);
    }
}
