//! Makepad application registration, startup and event dispatch.

use super::*;
#[cfg(all(windows, feature = "toolkit-uia"))]
use crate::accessibility::AccessibilityUiAction;

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        self.settings = DesktopSettings::load();
        // Styling is application-owned state. It deliberately never participates in
        // document snapshots or GLYF/GLYP serialization.
        self.style_profile = self.settings.style_profile;
        self.panel_resources = self.settings.panel_resources;
        self.panel_layers = self.settings.panel_layers && !self.panel_resources;
        self.settings_category = SettingsCategory::General;
        self.context_menu_open = false;
        self.overlay_pos = None;
        self.project_path = "project.glyf".into();
        self.status_text = "Ready — create a project or add an image".into();
        self.ensure_active_tab();
        self.refresh(cx);
        self.apply_desktop_style(cx);
        self.sync_shell_labels(cx);
        self.set_status(cx, self.status_text.clone());
        let cli_path = std::env::args()
            .nth(1)
            .filter(|path| !path.starts_with('-') && path.to_ascii_lowercase().ends_with(".glyf"));
        if let Some(path) = cli_path {
            self.project_path = path;
            self.open_project(cx);
        } else if self.settings.restore_session {
            self.open_recent(cx);
        }
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        #[cfg(all(windows, feature = "toolkit-uia"))]
        for action in actions
            .iter()
            .filter_map(|action| action.downcast_ref::<AccessibilityUiAction>())
            .map(|action| action.0.clone())
            .collect::<Vec<_>>()
        {
            self.handle_accessibility_action(cx, action);
        }
        if actions
            .iter()
            .filter_map(|action| action.downcast_ref::<OverlayHostAction>())
            .any(|action| matches!(action, OverlayHostAction::Dismissed))
        {
            self.close_menu(cx);
        }
        let tab_strip = self.ui.widget(cx, ids!(tabs));
        for action in actions.filter_widget_actions(tab_strip.widget_uid()) {
            self.handle_document_tab_action(cx, action.cast());
        }
        if self.ui.button(cx, ids!(tab_new)).clicked(actions) {
            self.new_document_tab(cx);
        }
        if self.ui.button(cx, ids!(tab_overflow)).clicked(actions) {
            self.open_tab_overflow(cx);
        }
        if self.ui.button(cx, ids!(chrome_minimize)).clicked(actions) {
            #[cfg(windows)]
            minimize_editor_window();
        }
        if self.ui.button(cx, ids!(chrome_maximize)).clicked(actions) {
            #[cfg(windows)]
            toggle_editor_window_maximized();
        }
        if self.ui.button(cx, ids!(chrome_close)).clicked(actions) {
            self.request_editor_exit(cx);
        }
        if self.ui.button(cx, ids!(menu_file)).clicked(actions) {
            self.toggle_menu(cx, "file");
        }
        if self.ui.button(cx, ids!(menu_edit)).clicked(actions) {
            self.toggle_menu(cx, "edit");
        }
        if self.ui.button(cx, ids!(menu_view)).clicked(actions) {
            self.toggle_menu(cx, "view");
        }
        if self.ui.button(cx, ids!(menu_window)).clicked(actions) {
            self.toggle_menu(cx, "window");
        }
        if self.ui.button(cx, ids!(menu_language)).clicked(actions) {
            self.toggle_menu(cx, "language");
        }
        if self.ui.button(cx, ids!(menu_help)).clicked(actions) {
            self.toggle_menu(cx, "help");
        }
        if self.ui.button(cx, ids!(menu_file_new)).clicked(actions) {
            self.new_document_tab(cx);
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(menu_file_open)).clicked(actions) {
            self.open_project_action(cx);
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(menu_file_recent)).clicked(actions) {
            self.open_recent_submenu(cx);
        }
        for (index, id) in [
            ids!(recent_file_0),
            ids!(recent_file_1),
            ids!(recent_file_2),
            ids!(recent_file_3),
            ids!(recent_file_4),
            ids!(recent_file_5),
            ids!(recent_file_6),
            ids!(recent_file_7),
            ids!(recent_file_8),
            ids!(recent_file_9),
        ]
        .into_iter()
        .enumerate()
        {
            if self.ui.button(cx, id).clicked(actions) {
                if let Some(path) = desktop::recent_menu_entries(&self.settings.recent_files)
                    .get(index)
                    .map(|entry| entry.path.clone())
                {
                    self.open_recent_path(cx, &path);
                }
            }
        }
        if self.ui.button(cx, ids!(recent_clear)).clicked(actions) {
            self.clear_recent(cx);
        }
        if self.ui.button(cx, ids!(menu_file_save)).clicked(actions) {
            self.save_project_command(cx);
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(menu_file_save_as)).clicked(actions) {
            self.save_project_as(cx);
            self.close_menu(cx);
        }
        if self
            .ui
            .button(cx, ids!(menu_file_save_pack))
            .clicked(actions)
        {
            self.save_project(cx);
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(menu_file_close)).clicked(actions) {
            self.close_active_tab(cx);
        }
        if self.ui.button(cx, ids!(menu_file_exit)).clicked(actions) {
            if self.dirty {
                self.set_status(cx, "Unsaved changes — save before Exit");
                self.close_menu(cx);
            } else {
                cx.quit();
            }
        }
        if self.ui.button(cx, ids!(menu_edit_undo)).clicked(actions) {
            self.undo(cx);
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(menu_edit_redo)).clicked(actions) {
            self.redo(cx);
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(menu_edit_cut)).clicked(actions) {
            self.cut_selection(cx);
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(menu_edit_copy)).clicked(actions) {
            self.copy_selection(cx);
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(menu_edit_paste)).clicked(actions) {
            self.paste_selection(cx);
            self.close_menu(cx);
        }
        if self
            .ui
            .button(cx, ids!(menu_edit_duplicate))
            .clicked(actions)
        {
            self.duplicate_selection(cx);
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(menu_edit_delete)).clicked(actions) {
            if self.selected_annotation.is_some() {
                self.delete_annotation(cx);
            } else {
                self.remove_part(cx);
            }
            self.close_menu(cx);
        }
        if self
            .ui
            .button(cx, ids!(menu_edit_select_all))
            .clicked(actions)
        {
            self.select_all(cx);
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(context_copy)).clicked(actions) {
            self.copy_selection(cx);
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(context_cut)).clicked(actions) {
            self.cut_selection(cx);
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(context_paste)).clicked(actions) {
            self.paste_selection(cx);
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(context_duplicate)).clicked(actions) {
            self.duplicate_selection(cx);
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(context_delete)).clicked(actions) {
            if self.selected_annotation.is_some() {
                self.delete_annotation(cx);
            } else {
                self.remove_part(cx);
            }
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(menu_settings)).clicked(actions) {
            self.open_shell_panel(cx, "settings");
        }
        if self.ui.button(cx, ids!(menu_view_zoom_in)).clicked(actions) {
            self.adjust_canvas_zoom(cx, 1.25);
            self.close_menu(cx);
        }
        if self
            .ui
            .button(cx, ids!(menu_view_zoom_out))
            .clicked(actions)
        {
            self.adjust_canvas_zoom(cx, 0.8);
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(menu_view_fit)).clicked(actions) {
            self.fit_canvas(cx);
            self.close_menu(cx);
        }
        if self
            .ui
            .button(cx, ids!(menu_view_resources))
            .clicked(actions)
        {
            self.toggle_sidebar(cx, "resources");
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(menu_view_layers)).clicked(actions) {
            self.toggle_sidebar(cx, "layers");
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(menu_window_next)).clicked(actions) {
            self.next_tab(cx);
            self.close_menu(cx);
        }
        if self
            .ui
            .button(cx, ids!(menu_window_previous))
            .clicked(actions)
        {
            self.previous_tab(cx);
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(menu_window_close)).clicked(actions) {
            self.close_active_tab(cx);
        }
        if self
            .ui
            .button(cx, ids!(menu_window_close_others))
            .clicked(actions)
        {
            self.close_other_tabs(cx);
            self.close_menu(cx);
        }
        if self
            .ui
            .button(cx, ids!(menu_window_close_all))
            .clicked(actions)
        {
            self.close_all_tabs(cx);
            self.close_menu(cx);
        }
        if self
            .ui
            .button(cx, ids!(menu_language_english))
            .clicked(actions)
        {
            self.set_language(cx, Language::English);
            self.close_menu(cx);
        }
        if self
            .ui
            .button(cx, ids!(menu_language_russian))
            .clicked(actions)
        {
            self.set_language(cx, Language::Russian);
            self.close_menu(cx);
        }
        if self.ui.button(cx, ids!(menu_help_about)).clicked(actions) {
            self.open_shell_panel(cx, "about");
        }
        if self
            .ui
            .button(cx, ids!(menu_help_shortcuts))
            .clicked(actions)
        {
            self.open_shell_panel(cx, "shortcuts");
        }
        if self.ui.button(cx, ids!(settings_close)).clicked(actions)
            || self.ui.button(cx, ids!(about_close)).clicked(actions)
            || self.ui.button(cx, ids!(shortcuts_close)).clicked(actions)
        {
            self.close_menu(cx);
        }
        if self
            .ui
            .button(cx, ids!(close_confirm_cancel))
            .clicked(actions)
        {
            self.cancel_tab_close(cx);
        }
        if self
            .ui
            .button(cx, ids!(close_confirm_dont_save))
            .clicked(actions)
        {
            self.discard_tab_close(cx);
        }
        if self
            .ui
            .button(cx, ids!(close_confirm_save))
            .clicked(actions)
        {
            self.save_and_close_tab(cx);
        }
        for (category, id) in [
            (SettingsCategory::General, ids!(settings_category_general)),
            (SettingsCategory::Appearance, ids!(settings_category_appearance)),
            (SettingsCategory::Editor, ids!(settings_category_editor)),
            (SettingsCategory::Files, ids!(settings_category_files)),
            (
                SettingsCategory::Accessibility,
                ids!(settings_category_accessibility),
            ),
        ] {
            if self.ui.button(cx, id).clicked(actions) {
                self.set_settings_category(cx, category);
            }
        }
        if self.ui.button(cx, ids!(settings_restore)).clicked(actions) {
            self.toggle_restore_session(cx);
        }
        if self.ui.button(cx, ids!(settings_reopen_last)).clicked(actions) {
            self.toggle_restore_session(cx);
        }
        if let Some(index) = self.ui.drop_down(cx, ids!(settings_language_select)).selected(actions) {
            self.set_language(cx, if index == 1 { Language::Russian } else { Language::English });
        }
        let mut settings_changed = false;
        let mut appearance_changed = false;
        if let Some(index) = self.ui.drop_down(cx, ids!(settings_recent_count)).selected(actions) {
            self.settings.recent_file_count = if index == 0 { 5 } else { 10 };
            self.settings.recent_files.truncate(self.settings.recent_file_count);
            settings_changed = true;
        }
        if let Some(index) = self.ui.drop_down(cx, ids!(settings_preset)).selected(actions) {
            if let Some(preset) = StylePreset::ALL.get(index) {
                self.style_profile = StyleProfile::from_preset(*preset);
                settings_changed = true;
                appearance_changed = true;
            }
        }
        if let Some(index) = self.ui.drop_down(cx, ids!(settings_palette)).selected(actions) {
            if let Some(value) = PaletteStyle::ALL.get(index) {
                self.style_profile.palette = *value;
                settings_changed = true;
                appearance_changed = true;
            }
        }
        if let Some(index) = self.ui.drop_down(cx, ids!(settings_shape)).selected(actions) {
            if let Some(value) = ShapeStyle::ALL.get(index) {
                self.style_profile.shape = *value;
                settings_changed = true;
                appearance_changed = true;
            }
        }
        if let Some(index) = self.ui.drop_down(cx, ids!(settings_surface)).selected(actions) {
            if let Some(value) = SurfaceStyle::ALL.get(index) {
                self.style_profile.surface = *value;
                settings_changed = true;
                appearance_changed = true;
            }
        }
        if let Some(index) = self.ui.drop_down(cx, ids!(settings_motion)).selected(actions) {
            if let Some(value) = MotionStyle::ALL.get(index) {
                self.style_profile.motion = *value;
                settings_changed = true;
                appearance_changed = true;
            }
        }
        if let Some(index) = self.ui.drop_down(cx, ids!(settings_font_size)).selected(actions) {
            if let Some(value) = FontSizeChoice::ALL.get(index) {
                self.settings.font_size = *value;
                settings_changed = true;
                appearance_changed = true;
            }
        }
        if let Some(index) = self.ui.drop_down(cx, ids!(settings_ui_scale)).selected(actions) {
            if let Some(value) = UiScaleChoice::ALL.get(index) {
                self.settings.ui_scale = *value;
                settings_changed = true;
                appearance_changed = true;
            }
        }
        if let Some(index) = self.ui.drop_down(cx, ids!(settings_density)).selected(actions) {
            if let Some(value) = DensityChoice::ALL.get(index) {
                self.settings.density = *value;
                settings_changed = true;
                appearance_changed = true;
            }
        }
        if let Some(index) = self.ui.drop_down(cx, ids!(settings_default_zoom)).selected(actions) {
            if let Some(value) = [0.5, 0.75, 1.0, 1.25, 1.5].get(index) {
                self.settings.default_zoom = *value;
                settings_changed = true;
            }
        }
        if self.ui.button(cx, ids!(settings_resources)).clicked(actions) {
            self.toggle_sidebar(cx, "resources");
        }
        if self.ui.button(cx, ids!(settings_layers)).clicked(actions) {
            self.toggle_sidebar(cx, "layers");
        }
        if self.ui.button(cx, ids!(settings_status_bar)).clicked(actions) {
            self.settings.panel_status_bar = !self.settings.panel_status_bar;
            settings_changed = true;
            appearance_changed = true;
        }
        if self.ui.button(cx, ids!(settings_confirm_destructive)).clicked(actions) {
            self.settings.confirm_destructive = !self.settings.confirm_destructive;
            settings_changed = true;
        }
        if self.ui.button(cx, ids!(settings_reduced_motion)).clicked(actions) {
            self.style_profile.motion = if matches!(self.style_profile.motion, MotionStyle::None | MotionStyle::Reduced) {
                MotionStyle::Subtle
            } else {
                MotionStyle::Reduced
            };
            settings_changed = true;
            appearance_changed = true;
        }
        if self.ui.button(cx, ids!(settings_large_font)).clicked(actions) {
            self.settings.font_size = if self.settings.font_size == FontSizeChoice::Large {
                FontSizeChoice::Normal
            } else {
                FontSizeChoice::Large
            };
            settings_changed = true;
            appearance_changed = true;
        }
        if self.ui.button(cx, ids!(settings_reset_appearance)).clicked(actions) {
            let defaults = DesktopSettings::default();
            self.style_profile = defaults.style_profile;
            self.settings.font_size = defaults.font_size;
            self.settings.ui_scale = defaults.ui_scale;
            self.settings.density = defaults.density;
            settings_changed = true;
            appearance_changed = true;
        }
        if self.ui.button(cx, ids!(settings_reset_all)).clicked(actions) {
            self.ui.widget(cx, ids!(settings_reset_confirm)).set_visible(cx, true);
        }
        if self.ui.button(cx, ids!(settings_reset_cancel)).clicked(actions) {
            self.ui.widget(cx, ids!(settings_reset_confirm)).set_visible(cx, false);
        }
        if self.ui.button(cx, ids!(settings_reset_yes)).clicked(actions) {
            self.settings = DesktopSettings::default();
            self.style_profile = self.settings.style_profile;
            self.panel_resources = self.settings.panel_resources;
            self.panel_layers = self.settings.panel_layers;
            self.ui.widget(cx, ids!(settings_reset_confirm)).set_visible(cx, false);
            settings_changed = true;
            appearance_changed = true;
        }
        if settings_changed {
            self.commit_desktop_settings(cx, appearance_changed);
        }

        if self.ui.button(cx, ids!(new_project)).clicked(actions) {
            self.new_project(cx);
        }
        if self.ui.button(cx, ids!(save_project)).clicked(actions) {
            self.save_project_command(cx);
        }
        if self.ui.button(cx, ids!(sidebar_images)).clicked(actions) {
            self.toggle_sidebar(cx, "resources");
        }
        if self.ui.button(cx, ids!(sidebar_layers)).clicked(actions) {
            self.toggle_sidebar(cx, "layers");
        }
        if self.ui.button(cx, ids!(add_image)).clicked(actions) {
            self.add_image_action(cx);
        }
        if self.ui.button(cx, ids!(remove_image)).clicked(actions) {
            self.remove_image(cx);
        }
        if self.ui.button(cx, ids!(update_image)).clicked(actions) {
            self.update_image(cx);
        }
        if self.ui.button(cx, ids!(edit_source)).clicked(actions) {
            self.enter_source_mode(cx);
        }

        for action in actions
            .iter()
            .filter_map(|action| action.downcast_ref::<AnnotationCanvasAction>())
            .cloned()
            .collect::<Vec<_>>()
        {
            self.handle_annotation_canvas_action(cx, action);
        }
        for action in actions
            .iter()
            .filter_map(|action| action.downcast_ref::<CanvasViewportAction>())
            .cloned()
            .collect::<Vec<_>>()
        {
            self.handle_canvas_viewport_action(cx, action);
        }
        for action in actions
            .iter()
            .filter_map(|action| action.downcast_ref::<CanvasHoverAction>())
            .cloned()
            .collect::<Vec<_>>()
        {
            self.handle_canvas_hover_action(cx, action);
        }
        for action in actions
            .iter()
            .filter_map(|action| action.downcast_ref::<CanvasEditAction>())
            .cloned()
            .collect::<Vec<_>>()
        {
            self.handle_canvas_edit_action(cx, action);
        }
        for action in actions
            .iter()
            .filter_map(|action| action.downcast_ref::<SourceCanvasAction>())
            .cloned()
            .collect::<Vec<_>>()
        {
            self.handle_source_canvas_action(cx, action);
        }
        let (clip_commit, clip_cancel, clip_name_returned, clip_comment_returned) = self
            .ui
            .widget(cx, ids!(clip_popup))
            .borrow_mut::<ClipPopup>()
            .map(|mut popup| {
                (
                    popup.commit_clicked(cx, actions),
                    popup.cancel_clicked(cx, actions),
                    popup.name_returned(cx, actions),
                    popup.comment_returned(cx, actions),
                )
            })
            .unwrap_or((false, false, false, false));
        if clip_commit || clip_name_returned || clip_comment_returned {
            if self.pending_part_popup.is_some() {
                self.commit_part_popup(cx);
            } else {
                self.commit_clip(cx);
            }
        }
        if clip_cancel {
            if self.pending_part_popup.is_some() {
                self.cancel_part_popup(cx);
            } else {
                self.cancel_clip(cx);
            }
        }
        if self.pending_annotation.is_some() {
            self.annotation_draft = self
                .ui
                .text_input(cx, ids!(annotation_popup.comment_input))
                .text();
        }
        let popup_commit = self
            .ui
            .button(cx, ids!(annotation_popup.actions.commit))
            .clicked(actions);
        let popup_returned = self
            .ui
            .text_input(cx, ids!(annotation_popup.comment_input))
            .returned(actions)
            .is_some();
        let popup_cancel = self
            .ui
            .button(cx, ids!(annotation_popup.actions.cancel))
            .clicked(actions);
        let popup_escaped = self
            .ui
            .text_input(cx, ids!(annotation_popup.comment_input))
            .escaped(actions);
        if popup_commit || popup_returned {
            self.commit_annotation(cx);
        }
        if popup_cancel || popup_escaped {
            self.cancel_annotation(cx);
        }
        for action in actions
            .iter()
            .filter_map(|action| action.downcast_ref::<SpatialCanvasAction>())
            .cloned()
            .collect::<Vec<_>>()
        {
            self.handle_canvas_action(cx, action);
        }
        if self.ui.button(cx, ids!(tool_select)).clicked(actions) {
            self.set_canvas_tool(cx, CanvasTool::Select);
        }
        if self.ui.button(cx, ids!(tool_point)).clicked(actions) {
            self.set_canvas_tool(cx, CanvasTool::Point);
        }
        if self.ui.button(cx, ids!(tool_region)).clicked(actions) {
            self.set_canvas_tool(cx, CanvasTool::Region);
        }
        if self.ui.button(cx, ids!(canvas_fit)).clicked(actions) {
            self.fit_canvas(cx);
        }
        if self.ui.button(cx, ids!(canvas_100)).clicked(actions) {
            self.canvas_100(cx);
        }
        if self.ui.button(cx, ids!(canvas_zoom_out)).clicked(actions) {
            self.adjust_canvas_zoom(cx, 0.8);
        }
        if self.ui.button(cx, ids!(canvas_zoom_in)).clicked(actions) {
            self.adjust_canvas_zoom(cx, 1.25);
        }
        if self.ui.button(cx, ids!(source_back)).clicked(actions) {
            self.leave_source_mode(cx);
        }
        if self.ui.button(cx, ids!(source_select)).clicked(actions) {
            self.set_source_tool(cx, SourceTool::Select);
        }
        if self.ui.button(cx, ids!(source_clip)).clicked(actions) {
            self.set_source_tool(cx, SourceTool::Clip);
        }
        if self.ui.button(cx, ids!(source_fit)).clicked(actions) {
            self.fit_source(cx);
        }
        if self.ui.button(cx, ids!(source_100)).clicked(actions) {
            self.source_100(cx);
        }
        if self.ui.button(cx, ids!(source_zoom_out)).clicked(actions) {
            self.adjust_source_zoom(cx, 0.8);
        }
        if self.ui.button(cx, ids!(source_zoom_in)).clicked(actions) {
            self.adjust_source_zoom(cx, 1.25);
        }
        if self.ui.button(cx, ids!(annotation_delete)).clicked(actions) {
            self.delete_annotation(cx);
        }
        if self.ui.button(cx, ids!(annotation_edit)).clicked(actions) {
            if let Some(index) = self.selected_annotation {
                self.begin_edit_annotation(cx, index);
            }
        }
        if let Some(index) = self
            .ui
            .drop_down(cx, ids!(annotation_dropdown))
            .selected(actions)
        {
            if index < self.annotations.len() {
                self.selected_annotation = Some(index);
                self.selected_part = None;
                self.sync_spatial(cx);

                self.set_status(cx, "Annotation selected");
            }
        }
        if self.ui.button(cx, ids!(part_add)).clicked(actions) {
            self.add_part(cx, false);
        }
        if self.ui.button(cx, ids!(part_add_clip)).clicked(actions) {
            self.add_part(cx, true);
        }
        if self.ui.button(cx, ids!(part_remove)).clicked(actions) {
            self.remove_part(cx);
        }
        if self.ui.button(cx, ids!(part_up)).clicked(actions) {
            self.move_part(cx, -1);
        }
        if self.ui.button(cx, ids!(part_down)).clicked(actions) {
            self.move_part(cx, 1);
        }
        if self.ui.button(cx, ids!(part_back)).clicked(actions) {
            self.move_part_to(cx, false);
        }
        if self.ui.button(cx, ids!(part_front)).clicked(actions) {
            self.move_part_to(cx, true);
        }
        if self.ui.button(cx, ids!(part_apply)).clicked(actions) {
            self.apply_part(cx);
        }
        if self.ui.button(cx, ids!(part_edit_clip)).clicked(actions) {
            if let Some(index) = self.selected_part {
                self.begin_edit_clip(cx, index);
            }
        }
        if let Some(index) = self.ui.drop_down(cx, ids!(part_dropdown)).selected(actions) {
            if index < self.current_parts().len() {
                self.selected_part = Some(index);
                self.selected_annotation = None;
                self.sync_spatial(cx);

                self.set_status(cx, "Part selected");
            }
        }
        if let Some(index) = self
            .ui
            .drop_down(cx, ids!(image_dropdown))
            .selected(actions)
        {
            if index < self.images.len() {
                self.selected = Some(index);
                self.load_preview(cx);
                self.refresh(cx);
                self.set_status(cx, "Image selected");
            }
        }
        #[cfg(all(windows, feature = "toolkit-uia"))]
        self.refresh_accessibility(cx);
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        // Register standard widgets and shared aliases before selecting the
        // application theme. Selecting an alias before it exists leaves
        // mod.theme incomplete and makes the shell render as an empty canvas.
        makepad_widgets::theme_mod(vm);
        makepad_widgets::widgets_mod(vm);
        designing_makepad::script_mod(vm);
        match DesktopSettings::load().theme {
            ThemeChoice::Light => {
                script_eval!(vm, { mod.theme = mod.themes.glyf_light });
            }
            ThemeChoice::System | ThemeChoice::Dark => {
                script_eval!(vm, { mod.theme = mod.themes.glyf_production });
            }
        }
        clip_popup::script_mod(vm);
        overlay_host::script_mod(vm);
        tab_strip::script_mod(vm);
        source::script_mod(vm);
        spatial::script_mod(vm);
        crate::ui::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.handle_close_request(cx, event);
        self.handle_global_key_event(cx, event);
        self.update_toolbar_tooltip(cx, event);
        if self.dismiss_shell_overlay_on_outside_pointer(cx, event) {
            return;
        }
        self.open_recent_submenu_on_hover(cx, event);
        if self.handle_custom_chrome_pointer(cx, event) {
            return;
        }
        if self.route_annotation_popup_pointer(cx, event) {
            return;
        }
        if self.handle_canvas_context_pointer(cx, event) {
            return;
        }
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
        if matches!(event, Event::Draw(_)) && self.annotation_popup_focus_on_next_draw {
            if self.pending_annotation.is_some() {
                self.ui
                    .text_input(cx, ids!(annotation_popup.comment_input))
                    .set_key_focus(cx);
            }
            self.annotation_popup_focus_on_next_draw = false;
        }
        #[cfg(all(windows, feature = "toolkit-uia"))]
        if matches!(event, Event::Draw(_)) {
            self.refresh_accessibility(cx);
        }
    }
}
