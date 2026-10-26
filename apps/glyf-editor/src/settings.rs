//! Desktop preferences, localization and style application.

use super::*;

impl App {
    pub(crate) fn set_settings_category(&mut self, cx: &mut Cx, category: SettingsCategory) {
        self.settings_category = category;
        self.sync_settings_category(cx);
        self.ui.widget(cx, ids!(settings_panel)).redraw(cx);
    }

    pub(crate) fn sync_settings_category(&mut self, cx: &mut Cx) {
        let entries = [
            (
                SettingsCategory::General,
                ids!(settings_category_general),
                ids!(settings_general_view),
                "settings.general",
            ),
            (
                SettingsCategory::Appearance,
                ids!(settings_category_appearance),
                ids!(settings_appearance_view),
                "settings.appearance",
            ),
            (
                SettingsCategory::Editor,
                ids!(settings_category_editor),
                ids!(settings_editor_view),
                "settings.editor",
            ),
            (
                SettingsCategory::Files,
                ids!(settings_category_files),
                ids!(settings_files_view),
                "settings.files",
            ),
            (
                SettingsCategory::Accessibility,
                ids!(settings_category_accessibility),
                ids!(settings_accessibility_view),
                "settings.accessibility",
            ),
        ];
        let tokens = self.style_profile.tokens(StyleTuning::default());
        for (category, button_id, view_id, key) in entries {
            let selected = self.settings_category == category;
            self.ui.widget(cx, view_id).set_visible(cx, selected);
            let label = desktop::text(key, self.settings.language);
            self.ui.button(cx, button_id).set_text(
                cx,
                &if selected {
                    format!("●  {label}")
                } else {
                    format!("    {label}")
                },
            );
            let fill = if selected {
                tokens.accent_subtle
            } else {
                tokens.panel_bg
            };
            let ink = if selected {
                tokens.accent
            } else {
                tokens.text_primary
            };
            let mut button = self.ui.button(cx, button_id);
            script_apply_eval!(cx, button, {
                draw_bg +: { color: #(fill) }
                draw_text +: { color: #(ink) }
            });
        }
    }

    pub(crate) fn sync_settings_controls(&mut self, cx: &mut Cx) {
        let language = self.settings.language;
        let choices = |english: &[&str], russian: &[&str]| -> Vec<String> {
            (if language == Language::Russian { russian } else { english })
                .iter().map(|value| (*value).to_owned()).collect()
        };
        for (id, english, russian) in [
            (ids!(settings_preset), &["Classic Light", "Midnight", "Neon", "Glass Dark", "Soft Light", "Sharp Dark", "Custom"][..], &["Классическая светлая", "Полночь", "Неон", "Тёмное стекло", "Мягкая светлая", "Строгая тёмная", "Своя"][..]),
            (ids!(settings_palette), &["Dark", "Light", "Neon"][..], &["Тёмная", "Светлая", "Неоновая"][..]),
            (ids!(settings_shape), &["Rounded", "Soft", "Sharp", "Angular"][..], &["Скруглённая", "Мягкая", "Строгая", "Угловатая"][..]),
            (ids!(settings_surface), &["Flat", "Elevated", "Glass", "Neumorphic"][..], &["Плоская", "Объёмная", "Стеклянная", "Неоморфная"][..]),
            (ids!(settings_motion), &["None", "Reduced", "Subtle", "Smooth"][..], &["Без анимации", "Уменьшенная", "Лёгкая", "Плавная"][..]),
            (ids!(settings_font_size), &["Small", "Normal", "Large"][..], &["Мелкий", "Обычный", "Крупный"][..]),
            (ids!(settings_density), &["Compact", "Comfortable"][..], &["Компактная", "Комфортная"][..]),
        ] {
            self.ui.drop_down(cx, id).set_labels(cx, choices(english, russian));
        }
        for (id, key) in [
            (ids!(settings_general_heading), "settings.general"),
            (ids!(settings_appearance_heading), "settings.appearance"),
            (ids!(settings_editor_heading), "settings.editor"),
            (ids!(settings_files_heading), "settings.files"),
            (ids!(settings_accessibility_heading), "settings.accessibility"),
            (ids!(settings_language_label), "settings.language"),
            (ids!(settings_recent_label), "settings.recent_count"),
            (ids!(settings_preset_label), "settings.preset"),
            (ids!(settings_palette_label), "settings.palette"),
            (ids!(settings_shape_label), "settings.shape"),
            (ids!(settings_surface_label), "settings.surface"),
            (ids!(settings_motion_label), "settings.motion"),
            (ids!(settings_font_label), "settings.font_size"),
            (ids!(settings_scale_label), "settings.ui_scale"),
            (ids!(settings_density_label), "settings.density"),
            (ids!(settings_zoom_label), "settings.default_zoom"),
            (ids!(settings_reset_confirm_label), "settings.reset_all"),
        ] {
            self.ui.label(cx, id).set_text(cx, desktop::text(key, language));
        }
        for (id, key) in [
            (ids!(settings_reset_appearance), "settings.reset_appearance"),
            (ids!(settings_reset_all), "settings.reset_all"),
        ] {
            self.ui.button(cx, id).set_text(cx, desktop::text(key, language));
        }
        self.ui.button(cx, ids!(settings_reset_yes)).set_text(
            cx,
            if language == Language::Russian { "Сбросить" } else { "Reset" },
        );
        self.ui.button(cx, ids!(settings_reset_cancel)).set_text(
            cx,
            if language == Language::Russian { "Отмена" } else { "Cancel" },
        );
        self.ui
            .drop_down(cx, ids!(settings_language_select))
            .set_selected_item(cx, usize::from(language == Language::Russian));
        self.ui
            .drop_down(cx, ids!(settings_recent_count))
            .set_selected_item(cx, usize::from(self.settings.recent_file_count > 5));
        let preset_index = self
            .settings
            .style_profile;
        let preset_index = StylePreset::ALL
            .iter()
            .position(|preset| StyleProfile::from_preset(*preset) == preset_index)
            .unwrap_or(StylePreset::ALL.len());
        self.ui
            .drop_down(cx, ids!(settings_preset))
            .set_selected_item(cx, preset_index);
        for (id, selected) in [
            (
                ids!(settings_palette),
                PaletteStyle::ALL.iter().position(|value| *value == self.style_profile.palette),
            ),
            (
                ids!(settings_shape),
                ShapeStyle::ALL.iter().position(|value| *value == self.style_profile.shape),
            ),
            (
                ids!(settings_surface),
                SurfaceStyle::ALL.iter().position(|value| *value == self.style_profile.surface),
            ),
            (
                ids!(settings_motion),
                MotionStyle::ALL.iter().position(|value| *value == self.style_profile.motion),
            ),
            (
                ids!(settings_font_size),
                FontSizeChoice::ALL.iter().position(|value| *value == self.settings.font_size),
            ),
            (
                ids!(settings_ui_scale),
                UiScaleChoice::ALL.iter().position(|value| *value == self.settings.ui_scale),
            ),
            (
                ids!(settings_density),
                DensityChoice::ALL.iter().position(|value| *value == self.settings.density),
            ),
        ] {
            self.ui.drop_down(cx, id).set_selected_item(cx, selected.unwrap_or(0));
        }
        let zoom_index = [0.5, 0.75, 1.0, 1.25, 1.5]
            .iter()
            .position(|zoom| (self.settings.default_zoom - zoom).abs() < 0.01)
            .unwrap_or(2);
        self.ui
            .drop_down(cx, ids!(settings_default_zoom))
            .set_selected_item(cx, zoom_index);
        for (id, key, enabled) in [
            (ids!(settings_restore), "settings.restore_session", self.settings.restore_session),
            (ids!(settings_reopen_last), "settings.restore_session", self.settings.restore_session),
            (ids!(settings_resources), "settings.resources", self.panel_resources),
            (ids!(settings_layers), "settings.layers", self.panel_layers),
            (ids!(settings_status_bar), "settings.status_bar", self.settings.panel_status_bar),
            (ids!(settings_confirm_destructive), "settings.confirm_destructive", self.settings.confirm_destructive),
            (ids!(settings_reduced_motion), "settings.reduced_motion", matches!(self.style_profile.motion, MotionStyle::None | MotionStyle::Reduced)),
            (ids!(settings_large_font), "settings.font_size", self.settings.font_size == FontSizeChoice::Large),
        ] {
            let value = if enabled { "✓" } else { "○" };
            self.ui.button(cx, id).set_text(cx, &format!("{value}  {}", desktop::text(key, language)));
        }
        self.ui.button(cx, ids!(settings_inspector)).set_enabled(cx, false);
        self.ui.button(cx, ids!(settings_inspector)).set_text(cx,
            if language == Language::Russian { "Инспектор: убран из этой компоновки" } else { "Inspector: removed from this layout" });
        self.ui.button(cx, ids!(settings_glyp_behavior)).set_enabled(cx, false);
        self.ui.button(cx, ids!(settings_glyp_behavior)).set_text(cx,
            if language == Language::Russian { "GLYP: ручная упаковка через Файл" } else { "GLYP: manual pack from File" });
        self.ui.label(cx, ids!(settings_high_contrast_note)).set_text(cx,
            if language == Language::Russian { "Высокая контрастность: исследуется" } else { "High Contrast: under investigation" });
        self.ui.label(cx, ids!(settings_project_location_label)).set_text(cx,
            if language == Language::Russian { "Папка проекта: последняя выбранная" } else { "Project folder: last selected" });
    }

    pub(crate) fn sync_editor_labels(&mut self, cx: &mut Cx) {
        let language = self.settings.language;
        for (id, key) in [
            (ids!(resources_heading), "editor.images_heading"),
            (ids!(metadata_heading), "editor.metadata_heading"),
            (ids!(layers_heading), "editor.layers_heading"),
            (ids!(layers_hint), "editor.layers_hint"),
            (ids!(annotations_heading), "editor.annotations_heading"),
            (ids!(annotations_hint), "editor.annotations_hint"),
            (ids!(preview_placeholder), "editor.preview_empty"),
        ] {
            self.ui.label(cx, id).set_text(cx, desktop::text(key, language));
        }
        self.ui.label(cx, ids!(annotation_popup.title)).set_text(cx,
            desktop::text("editor.add_annotation", language));
        self.ui.label(cx, ids!(context_caption)).set_text(cx,
            if language == Language::Russian { "Контекстное меню" } else { "Context menu" });
        for (id, key) in [
            (ids!(add_image), "editor.add_image"),
            (ids!(remove_image), "editor.remove_image"),
            (ids!(update_image), "editor.update_resource"),
            (ids!(edit_source), "editor.open_source"),
            (ids!(part_add), "editor.add_part"),
            (ids!(part_add_clip), "editor.add_clip"),
            (ids!(part_remove), "editor.delete_part"),
            (ids!(part_edit_clip), "editor.edit_source_crop"),
            (ids!(part_up), "editor.up"),
            (ids!(part_down), "editor.down"),
            (ids!(part_back), "editor.back"),
            (ids!(part_front), "editor.front"),
            (ids!(annotation_edit), "editor.edit_annotation"),
            (ids!(annotation_delete), "editor.delete_annotation"),
            (ids!(context_copy), "copy"),
            (ids!(context_cut), "cut"),
            (ids!(context_paste), "paste"),
            (ids!(context_duplicate), "duplicate"),
            (ids!(context_delete), "delete"),
        ] {
            self.ui.button(cx, id).set_text(cx, desktop::text(key, language));
        }
        self.ui.button(cx, ids!(annotation_popup.actions.cancel)).set_text(cx,
            desktop::text("editor.cancel", language));
        self.ui.button(cx, ids!(annotation_popup.actions.commit)).set_text(cx,
            desktop::text("editor.done", language));
        for (id, key) in [
            (ids!(image_path_input), "editor.image_path_placeholder"),
            (ids!(name_input), "editor.image_name_placeholder"),
            (ids!(comment_input), "editor.resource_comment_placeholder"),
        ] {
            self.ui.text_input(cx, id).set_empty_text(cx, desktop::text(key, language).to_owned());
        }
        self.ui.text_input(cx, ids!(annotation_popup.comment_input)).set_empty_text(cx,
            desktop::text("editor.annotation_prompt", language).to_owned());

        if self.images.is_empty() {
            self.ui.label(cx, ids!(images_list)).set_text(cx, desktop::text("editor.no_images_yet", language));
            self.ui.drop_down(cx, ids!(image_dropdown)).set_labels(cx, vec![desktop::text("editor.no_images", language).to_owned()]);
        }
        if self.current_parts().is_empty() {
            self.ui.label(cx, ids!(layers_list)).set_text(cx, desktop::text("editor.preview_empty", language));
            self.ui.drop_down(cx, ids!(part_dropdown)).set_labels(cx, vec![desktop::text("editor.no_parts", language).to_owned()]);
        }
        if self.annotations.is_empty() {
            self.ui.label(cx, ids!(annotations_list)).set_text(cx, desktop::text("editor.no_annotations_yet", language));
            self.ui.drop_down(cx, ids!(annotation_dropdown)).set_labels(cx, vec![desktop::text("editor.no_annotations", language).to_owned()]);
        }
        let source = self.selected.and_then(|index| self.images.get(index)).map(|image| image.source.as_str()).unwrap_or("—");
        self.ui.label(cx, ids!(source_label)).set_text(cx, &format!("{} {source}", desktop::text("editor.source_label", language)));
        self.ui.label(cx, ids!(mode_badge)).set_text(cx, &if self.source_mode {
            format!("{} · {}", desktop::text("editor.source_mode", language), self.selected_image_name())
        } else {
            desktop::text("editor.composition", language).to_owned()
        });
        self.ui.label(cx, ids!(dirty_badge)).set_text(cx, desktop::text(if self.dirty { "editor.unsaved" } else { "editor.saved" }, language));
        let selected_annotation = self.selected_annotation.and_then(|index| self.annotations.get(index));
        self.ui.label(cx, ids!(annotation_geometry)).set_text(cx, &selected_annotation
            .map(|annotation| localized_annotation_geometry_summary(annotation, language))
            .unwrap_or_else(|| desktop::text("editor.no_selection", language).to_owned()));
        self.ui.label(cx, ids!(annotation_role)).set_text(cx, &format!("{} {}", desktop::text("editor.role_label", language), selected_annotation.map(|annotation| annotation.role.as_str()).unwrap_or("—")));
        self.ui.label(cx, ids!(annotation_rel)).set_text(cx, &format!("{} {}", desktop::text("editor.coordinates_label", language), selected_annotation.map(annotation_coordinates_summary).unwrap_or_else(|| "—".into())));
        self.ui.label(cx, ids!(annotation_comment)).set_text(cx, &format!("{} {}", desktop::text("editor.comment_label", language), selected_annotation.map(|annotation| annotation.comment.as_str()).unwrap_or("—")));
        self.sync_source_canvas(cx);
        self.ui.label(cx, ids!(status)).set_text(cx, &desktop::localized_status(&self.status_text, language));
    }

    pub(crate) fn set_language(&mut self, cx: &mut Cx, language: Language) {
        self.settings.language = language;
        if let Err(error) = self.settings.save() {
            self.set_status(cx, format!("Settings save failed: {error}"));
        }
        if self.menu_open.as_deref() != Some("settings") {
            self.close_menu(cx);
        }
        self.sync_shell_labels(cx);
        self.set_status(
            cx,
            if language == Language::Russian {
                "Язык интерфейса: Русский"
            } else {
                "Interface language: English"
            },
        );
    }

    pub(crate) fn apply_desktop_style(&mut self, cx: &mut Cx) {
        shell_style::apply_makepad_palette(cx, self.style_profile);
        let tokens = shell_style::shell_style_tokens(self.style_profile, StyleTuning::default());
        // Rounded shell panels expose the window pass in the gaps between them.
        // Keep that pass in the active palette instead of Makepad's dark default.
        let window_clear = tokens.window.fill;
        let mut window = self.ui.widget(cx, ids!(main_window));
        script_apply_eval!(cx, window, { pass +: { clear_color: #(window_clear) } });
        for (id, color) in [
            (ids!(main_layout), tokens.window.fill),
            (ids!(chrome_bar), tokens.toolbar.fill),
            (ids!(menu_bar), tokens.toolbar.fill),
            (ids!(toolbar), tokens.toolbar.fill),
            (ids!(sidebar), tokens.toolbar.fill),
            (ids!(preview), tokens.canvas.fill),
            (ids!(canvas_overlay), tokens.canvas.fill),
            (ids!(source_canvas), tokens.canvas.fill),
            (ids!(status_bar), tokens.status.fill),
            (ids!(file_menu), tokens.menu.fill),
            (ids!(file_recent_submenu), tokens.menu.fill),
            (ids!(edit_menu), tokens.menu.fill),
            (ids!(view_menu), tokens.menu.fill),
            (ids!(language_menu), tokens.menu.fill),
            (ids!(help_menu), tokens.menu.fill),
            (ids!(context_menu), tokens.menu.fill),
            (ids!(settings_panel), tokens.menu.fill),
            (ids!(about_panel), tokens.menu.fill),
            (ids!(shortcuts_panel), tokens.menu.fill),
            (ids!(close_confirm_panel), tokens.menu.fill),
        ] {
            let mut widget = self.ui.widget(cx, id);
            script_apply_eval!(cx, widget, { draw_bg +: { color: #(color) } });
        }
        let mut tooltip_fill = tokens.toolbar.fill;
        tooltip_fill.w = 1.0;
        let tooltip_border = tokens.separator;
        let tooltip_text = tokens.text_primary;
        let tooltip = self.ui.tooltip(cx, ids!(toolbar_tooltip));
        let mut tooltip_content = tooltip.view(cx, ids!(content));
        script_apply_eval!(cx, tooltip_content, {
            draw_bg +: { color: #(tooltip_fill) border_color: #(tooltip_border) }
        });
        let mut tooltip_label = tooltip.label(cx, ids!(content.tooltip_label));
        script_apply_eval!(cx, tooltip_label, {
            draw_text +: { color: #(tooltip_text) }
        });
        // Changing mod.theme does not recolor widget instances already built
        // from the previous palette. Apply contrast-critical shell text to the
        // live widgets as well, including popup items that may be hidden now.
        let primary = tokens.text_primary;
        for id in [
            ids!(chrome_title), ids!(tab_overflow), ids!(tab_new),
            ids!(chrome_minimize), ids!(chrome_maximize), ids!(chrome_close),
            ids!(menu_file), ids!(menu_edit), ids!(menu_view),
            ids!(menu_window), ids!(menu_language), ids!(menu_help),
            ids!(menu_file_new), ids!(menu_file_open), ids!(menu_file_recent),
            ids!(menu_file_save), ids!(menu_file_save_as), ids!(menu_file_save_pack),
            ids!(menu_file_close), ids!(menu_file_exit),
            ids!(recent_file_0), ids!(recent_file_1), ids!(recent_file_2),
            ids!(recent_file_3), ids!(recent_file_4), ids!(recent_file_5),
            ids!(recent_file_6), ids!(recent_file_7), ids!(recent_file_8),
            ids!(recent_file_9), ids!(recent_clear),
            ids!(menu_edit_undo), ids!(menu_edit_redo), ids!(menu_edit_cut),
            ids!(menu_edit_copy), ids!(menu_edit_paste), ids!(menu_edit_duplicate),
            ids!(menu_edit_delete), ids!(menu_edit_select_all), ids!(menu_settings),
            ids!(menu_view_zoom_in), ids!(menu_view_zoom_out), ids!(menu_view_fit),
            ids!(menu_view_resources), ids!(menu_view_layers),
            ids!(menu_window_next), ids!(menu_window_previous),
            ids!(menu_window_close), ids!(menu_window_close_others),
            ids!(menu_window_close_all), ids!(menu_language_english),
            ids!(menu_language_russian), ids!(menu_help_about),
            ids!(menu_help_shortcuts), ids!(context_caption), ids!(context_copy),
            ids!(context_cut), ids!(context_paste), ids!(context_duplicate),
            ids!(context_delete), ids!(settings_title), ids!(settings_close),
            ids!(settings_general_heading), ids!(settings_appearance_heading),
            ids!(settings_editor_heading), ids!(settings_files_heading),
            ids!(settings_accessibility_heading), ids!(settings_language_label),
            ids!(settings_recent_label), ids!(settings_preset_label),
            ids!(settings_palette_label), ids!(settings_shape_label),
            ids!(settings_surface_label), ids!(settings_motion_label),
            ids!(settings_font_label), ids!(settings_scale_label),
            ids!(settings_density_label), ids!(settings_zoom_label),
            ids!(settings_project_location_label),
            ids!(settings_reset_appearance), ids!(settings_reset_all),
            ids!(settings_reset_confirm_label), ids!(settings_reset_yes),
            ids!(settings_reset_cancel), ids!(about_title), ids!(about_close),
            ids!(shortcuts_title), ids!(shortcuts_close),
            ids!(close_confirm_title), ids!(close_confirm_cancel),
            ids!(close_confirm_dont_save), ids!(close_confirm_save),
            ids!(resources_heading), ids!(metadata_heading),
            ids!(layers_heading), ids!(annotations_heading),
        ] {
            let mut widget = self.ui.widget(cx, id);
            script_apply_eval!(cx, widget, { draw_text +: { color: #(primary) } });
        }
        let secondary = tokens.text_secondary;
        for id in [
            ids!(status), ids!(canvas_zoom_label), ids!(source_zoom_label),
            ids!(source_status), ids!(dirty_badge), ids!(recent_empty),
            ids!(settings_style_profile), ids!(settings_high_contrast_note),
            ids!(preview_placeholder),
            ids!(images_list), ids!(source_label), ids!(layers_list),
            ids!(layers_hint), ids!(annotations_list), ids!(annotations_hint),
            ids!(annotation_geometry), ids!(annotation_role),
            ids!(annotation_rel), ids!(annotation_comment),
        ] {
            let mut label = self.ui.label(cx, id);
            script_apply_eval!(cx, label, { draw_text +: { color: #(secondary) } });
        }
        let command = self.style_profile.tokens(StyleTuning::default()).button(ButtonVariant::Secondary);
        let fill = command.fill;
        let hover = command.hover;
        let pressed = command.pressed;
        let border = command.border;
        let border_focus = command.border_focus;
        let command_text = command.text;
        for id in [
            ids!(new_project), ids!(save_project), ids!(sidebar_images), ids!(sidebar_layers),
            ids!(tool_select), ids!(tool_point), ids!(tool_region),
            ids!(canvas_fit), ids!(canvas_100), ids!(canvas_zoom_out), ids!(canvas_zoom_in),
            ids!(source_back), ids!(source_select), ids!(source_clip),
            ids!(source_fit), ids!(source_100), ids!(source_zoom_out), ids!(source_zoom_in),
        ] {
            let mut button = self.ui.button(cx, id);
            script_apply_eval!(cx, button, {
                draw_bg +: {
                    color: #(fill) color_hover: #(hover) color_down: #(pressed)
                    border_color: #(border) border_color_hover: #(border_focus)
                }
                draw_text +: { color: #(command_text) color_hover: #(command_text) }
            });
        }
        let disabled_text = self.style_profile.tokens(StyleTuning::default()).text_disabled;
        let disabled_fill = command.disabled;
        for id in [
            ids!(add_image), ids!(remove_image), ids!(update_image), ids!(edit_source),
            ids!(part_add), ids!(part_add_clip), ids!(part_remove), ids!(part_edit_clip),
            ids!(part_up), ids!(part_down), ids!(part_back), ids!(part_front),
            ids!(annotation_edit), ids!(annotation_delete),
            ids!(settings_restore), ids!(settings_resources), ids!(settings_layers),
            ids!(settings_inspector), ids!(settings_status_bar),
            ids!(settings_confirm_destructive), ids!(settings_reopen_last),
            ids!(settings_glyp_behavior), ids!(settings_reduced_motion),
            ids!(settings_large_font),
        ] {
            let mut button = self.ui.button(cx, id);
            script_apply_eval!(cx, button, {
                draw_bg +: {
                    color: #(fill) color_hover: #(hover) color_down: #(pressed)
                    color_disabled: #(disabled_fill) border_color: #(border)
                }
                draw_text +: {
                    color: #(command_text) color_hover: #(command_text)
                    color_disabled: #(disabled_text)
                }
            });
        }
        let input = self.style_profile.tokens(StyleTuning::default()).input();
        let input_fill = input.fill;
        let input_hover = input.hover;
        let input_border = input.border;
        let input_text = input.text;
        for id in [ids!(image_path_input), ids!(name_input), ids!(comment_input)] {
            let mut field = self.ui.text_input(cx, id);
            script_apply_eval!(cx, field, {
                draw_bg +: {
                    color: #(input_fill) color_empty: #(input_fill)
                    color_hover: #(input_hover) border_color: #(input_border)
                }
                draw_text +: {
                    color: #(input_text) color_empty: #(secondary)
                    color_empty_hover: #(secondary)
                }
            });
        }
        for id in [
            ids!(image_dropdown), ids!(part_dropdown), ids!(annotation_dropdown),
            ids!(settings_language_select), ids!(settings_recent_count),
            ids!(settings_preset), ids!(settings_palette), ids!(settings_shape),
            ids!(settings_surface), ids!(settings_motion), ids!(settings_font_size),
            ids!(settings_ui_scale), ids!(settings_density), ids!(settings_default_zoom),
        ] {
            let mut dropdown = self.ui.drop_down(cx, id);
            script_apply_eval!(cx, dropdown, {
                draw_bg +: { color: #(input_fill) border_color: #(input_border) }
                draw_text +: { color: #(input_text) }
            });
        }
        self.apply_tool_button_selection(cx);
        let mut badge = self.ui.label(cx, ids!(mode_badge));
        let accent = tokens.accent;
        script_apply_eval!(cx, badge, { draw_text +: { color: #(accent) } });
        let typography = self.settings.typography_tokens();
        let caption = typography.caption;
        let interface = typography.interface;
        let heading = typography.heading;
        for id in [ids!(status), ids!(canvas_zoom_label), ids!(source_zoom_label)] {
            let mut label = self.ui.label(cx, id);
            script_apply_eval!(cx, label, {
                draw_text +: { text_style +: { font_size: #(caption) } }
            });
        }
        for id in [
            ids!(settings_language_label), ids!(settings_recent_label),
            ids!(settings_preset_label), ids!(settings_palette_label),
            ids!(settings_shape_label), ids!(settings_surface_label),
            ids!(settings_motion_label), ids!(settings_font_label),
            ids!(settings_scale_label), ids!(settings_density_label),
            ids!(settings_zoom_label),
        ] {
            let mut label = self.ui.label(cx, id);
            script_apply_eval!(cx, label, {
                draw_text +: { text_style +: { font_size: #(interface) } }
            });
        }
        for id in [
            ids!(settings_general_heading), ids!(settings_appearance_heading),
            ids!(settings_editor_heading), ids!(settings_files_heading),
            ids!(settings_accessibility_heading),
        ] {
            let mut label = self.ui.label(cx, id);
            script_apply_eval!(cx, label, {
                draw_text +: { text_style +: { font_size: #(heading) } }
            });
        }
        let spacing = self.settings.spacing_tokens();
        let chrome_height = 42.0 * self.settings.ui_scale.percent() as f64 / 100.0;
        // Keep the text toolbar compact while respecting UI scale.
        let toolbar_height = (tokens.toolbar_height * self.settings.ui_scale.percent() as f64 / 100.0)
            .max(48.0);
        let status_height = (28.0 * self.settings.ui_scale.percent() as f64 / 100.0).max(24.0);
        let toolbar_gap = spacing.sm as f64;
        let mut chrome = self.ui.widget(cx, ids!(chrome_bar));
        script_apply_eval!(cx, chrome, { height: #(chrome_height) });
        let mut toolbar = self.ui.widget(cx, ids!(toolbar));
        script_apply_eval!(cx, toolbar, { height: #(toolbar_height) spacing: #(toolbar_gap) });
        let mut status = self.ui.widget(cx, ids!(status_bar));
        script_apply_eval!(cx, status, { height: #(status_height) });
        self.ui.widget(cx, ids!(status_bar)).set_visible(cx, self.settings.panel_status_bar);
        self.sync_settings_category(cx);
        self.ui.redraw(cx);
    }

    pub(crate) fn toggle_restore_session(&mut self, cx: &mut Cx) {
        self.settings.restore_session = !self.settings.restore_session;
        if let Err(error) = self.settings.save() {
            self.set_status(cx, format!("Settings save failed: {error}"));
            return;
        }
        self.sync_shell_labels(cx);
        self.set_status(
            cx,
            if self.settings.restore_session {
                "Session restore enabled"
            } else {
                "Session restore disabled"
            },
        );
    }

    pub(crate) fn commit_desktop_settings(&mut self, cx: &mut Cx, appearance_changed: bool) {
        self.settings.style_profile = self.style_profile;
        self.settings.theme = if self.style_profile.palette == PaletteStyle::Light {
            ThemeChoice::Light
        } else {
            ThemeChoice::Dark
        };
        if let Err(error) = self.settings.save() {
            self.set_status(cx, format!("Settings save failed: {error}"));
        }
        if appearance_changed {
            self.apply_desktop_style(cx);
        }
        self.sync_shell_labels(cx);
        self.ui.widget(cx, ids!(settings_panel)).redraw(cx);
    }
    #[cfg(windows)]
    pub(crate) fn set_settings_semantic_value(&mut self, cx: &mut Cx, id: &str, value: &str) -> bool {
        if self.menu_open.as_deref() != Some("settings") {
            return false;
        }
        let normalized = value.trim().to_ascii_lowercase();
        let mut appearance_changed = false;
        match id {
            "settings.general.language" if matches!(normalized.as_str(), "en" | "english" | "ru" | "русский") => {
                self.set_language(cx, Language::from_code(&normalized));
                return true;
            }
            "settings.general.recent_count" if matches!(normalized.as_str(), "5" | "10") => {
                self.settings.recent_file_count = normalized.parse().unwrap_or(10);
                self.settings.recent_files.truncate(self.settings.recent_file_count);
            }
            "settings.appearance.preset" => {
                let Some(preset) = StylePreset::ALL.iter().find(|preset| {
                    preset.code() == normalized || preset.label().eq_ignore_ascii_case(&normalized)
                }) else { return false };
                self.style_profile = StyleProfile::from_preset(*preset);
                appearance_changed = true;
            }
            "settings.appearance.palette" => {
                let Some(choice) = PaletteStyle::ALL.iter().find(|choice| choice.code() == normalized) else { return false };
                self.style_profile.palette = *choice;
                appearance_changed = true;
            }
            "settings.appearance.shape" => {
                let Some(choice) = ShapeStyle::ALL.iter().find(|choice| choice.code() == normalized) else { return false };
                self.style_profile.shape = *choice;
                appearance_changed = true;
            }
            "settings.appearance.surface" => {
                let Some(choice) = SurfaceStyle::ALL.iter().find(|choice| choice.code() == normalized) else { return false };
                self.style_profile.surface = *choice;
                appearance_changed = true;
            }
            "settings.appearance.motion" => {
                let Some(choice) = MotionStyle::ALL.iter().find(|choice| choice.code() == normalized) else { return false };
                self.style_profile.motion = *choice;
                appearance_changed = true;
            }
            "settings.appearance.font_size" => {
                let Some(choice) = FontSizeChoice::from_code(&normalized) else { return false };
                self.settings.font_size = choice;
                appearance_changed = true;
            }
            "settings.appearance.ui_scale" => {
                let Some(choice) = UiScaleChoice::from_percent(normalized.trim_end_matches('%')) else { return false };
                self.settings.ui_scale = choice;
                appearance_changed = true;
            }
            "settings.appearance.density" => {
                let Some(choice) = DensityChoice::from_code(&normalized) else { return false };
                self.settings.density = choice;
                appearance_changed = true;
            }
            "settings.editor.default_zoom" => {
                let Ok(percent) = normalized.trim_end_matches('%').parse::<u16>() else { return false };
                if ![50, 75, 100, 125, 150].contains(&percent) { return false; }
                self.settings.default_zoom = f64::from(percent) / 100.0;
            }
            _ => return false,
        }
        self.commit_desktop_settings(cx, appearance_changed);
        true
    }
}
