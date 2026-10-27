//! Application menus, recent files, panels and status presentation.

use super::*;

impl App {
    pub(crate) fn file_recent_submenu_open(&self) -> bool {
        self.menu_open.as_deref() == Some("file")
            && self.menu_chain.active_submenu() == Some("recent")
    }

    pub(crate) fn shell_menu_viewport(&self, cx: &Cx) -> Rect {
        let rect = self.ui.view(cx, ids!(main_layout)).area().rect(cx);
        if rect.size.x > 2.0 && rect.size.y > 2.0 {
            rect
        } else {
            self.ui.view(cx, ids!(preview)).area().rect(cx)
        }
    }

    pub(crate) fn position_shell_panel(&mut self, cx: &mut Cx, id: &[LiveId], pos: DVec2) {
        if let Some(mut panel) = self.ui.widget(cx, id).borrow_mut::<View>() {
            panel.walk.abs_pos = Some(pos);
            panel.redraw(cx);
        }
    }

    pub(crate) fn file_recent_parent_rect(&self, cx: &Cx, root_pos: DVec2) -> Rect {
        let rect = self.ui.button(cx, ids!(menu_file_recent)).area().rect(cx);
        let matches_root = rect.size.x > 2.0
            && rect.size.y > 2.0
            && (rect.pos.x - root_pos.x).abs() < 80.0
            && (rect.pos.y - root_pos.y).abs() < 220.0;
        if matches_root {
            rect
        } else {
            Rect {
                pos: dvec2(root_pos.x + 10.0, root_pos.y + 76.0),
                size: dvec2(340.0, 29.0),
            }
        }
    }

    pub(crate) fn file_recent_submenu_size(&self) -> DVec2 {
        let files = self.settings.recent_files.len().min(RECENT_MENU_CAPACITY);
        if files == 0 {
            return dvec2(420.0, 49.0);
        }
        let rows = files + 1; // entries plus Clear Recent
        dvec2(
            420.0,
            20.0 + rows as f64 * 29.0 + (rows.saturating_sub(1) as f64 * 4.0) + 13.0,
        )
    }

    pub(crate) fn file_recent_submenu_pos(&self, cx: &Cx, root_pos: DVec2) -> DVec2 {
        let parent = self.file_recent_parent_rect(cx, root_pos);
        let viewport = self.shell_menu_viewport(cx);
        let size = self.file_recent_submenu_size();
        let placement = place_submenu(
            MenuRect::new(parent.pos.x, parent.pos.y, parent.size.x, parent.size.y),
            (size.x, size.y),
            MenuRect::new(
                viewport.pos.x,
                viewport.pos.y,
                viewport.size.x,
                viewport.size.y,
            ),
            4.0,
        );
        dvec2(placement.rect.x, placement.rect.y)
    }

    pub(crate) fn open_recent_submenu(&mut self, cx: &mut Cx) {
        self.context_menu_open = false;
        if self.menu_chain.root() != Some(DesktopMenu::File) {
            self.menu_chain.open_root(DesktopMenu::File);
        }
        self.menu_open = Some("file".into());
        self.overlay_pos = self
            .menu_anchor_pos(cx, "file")
            .or_else(|| Some(self.shell_overlay_pos(cx, "file")));
        let pruned = self.settings.prune_missing_recent();
        if pruned {
            let _ = self.settings.save();
        }
        self.menu_chain.open_submenu("recent");
        self.sync_shell_labels(cx);
        if pruned {
            self.set_status(cx, "Removed unavailable recent files");
        }
        #[cfg(all(windows, feature = "toolkit-uia"))]
        self.refresh_accessibility(cx);
    }

    pub(crate) fn open_recent_submenu_on_hover(&mut self, cx: &mut Cx, event: &Event) {
        if self.menu_open.as_deref() != Some("file") {
            return;
        }
        let Event::MouseMove(pointer) = event else {
            return;
        };
        let root_pos = self
            .overlay_pos
            .unwrap_or_else(|| self.shell_overlay_pos(cx, "file"));
        if self
            .file_recent_parent_rect(cx, root_pos)
            .contains(pointer.abs)
        {
            if !self.file_recent_submenu_open() {
                self.open_recent_submenu(cx);
            }
            return;
        }
        if self.file_recent_submenu_open()
            && (Rect {
                pos: root_pos,
                size: self.shell_overlay_size("file"),
            })
            .contains(pointer.abs)
        {
            // Moving to an ordinary root item closes the child surface. Moving
            // diagonally from the parent toward the child stays outside this
            // root rectangle, preserving a simple pointer corridor for MVP.
            self.menu_chain.close_submenus();
            self.sync_shell_labels(cx);
            #[cfg(all(windows, feature = "toolkit-uia"))]
            self.refresh_accessibility(cx);
        }
    }

    pub(crate) fn clear_recent(&mut self, cx: &mut Cx) {
        if self.settings.recent_files.is_empty() {
            self.set_status(cx, "No recent files");
            return;
        }
        self.settings.clear_recent();
        let _ = self.settings.save();
        self.sync_shell_labels(cx);
        self.set_status(cx, "Recent files cleared");
        #[cfg(all(windows, feature = "toolkit-uia"))]
        self.refresh_accessibility(cx);
    }

    pub(crate) fn open_recent_path(&mut self, cx: &mut Cx, path: &str) {
        let glyf_path = with_extension(path, "glyf");
        if !glyf_path.is_file() {
            self.settings.remove_recent(path);
            self.settings
                .remove_recent(&glyf_path.display().to_string());
            let _ = self.settings.save();
            self.sync_shell_labels(cx);
            self.set_status(cx, "Recent file no longer exists");
            return;
        }
        let canonical = fs::canonicalize(&glyf_path).ok();
        if let Some(index) = self.tabs.iter().position(|tab| {
            let tab_path = Path::new(&tab.path);
            tab_path == glyf_path
                || canonical
                    .as_ref()
                    .zip(fs::canonicalize(tab_path).ok().as_ref())
                    .is_some_and(|(candidate, open)| candidate == open)
        }) {
            self.settings.add_recent(&glyf_path);
            let _ = self.settings.save();
            self.activate_tab(cx, index);
            if self.active_tab != index {
                return;
            }
            self.close_menu(cx);
            self.set_status(
                cx,
                format!("Activated already-open {}", glyf_path.display()),
            );
            return;
        }
        self.project_path = glyf_path.display().to_string();
        self.open_project(cx);
        self.close_menu(cx);
    }

    pub(crate) fn sync_shell_labels(&mut self, cx: &mut Cx) {
        let language = self.settings.language;
        let profile_summary = localized_style_profile_summary(self.style_profile, language);
        self.sync_editor_labels(cx);
        for (id, key) in [
            (ids!(menu_file), "file"),
            (ids!(menu_edit), "edit"),
            (ids!(menu_view), "view"),
            (ids!(menu_window), "window"),
            (ids!(menu_language), "language"),
            (ids!(menu_help), "help"),
            (ids!(menu_file_new), "new"),
            (ids!(menu_file_open), "open"),
            (ids!(menu_file_recent), "file.open_recent"),
            (ids!(menu_file_save), "save"),
            (ids!(menu_file_save_as), "save_as"),
            (ids!(menu_file_save_pack), "save_pack"),
            (ids!(menu_file_close), "close"),
            (ids!(menu_file_exit), "exit"),
            (ids!(menu_edit_undo), "undo"),
            (ids!(menu_edit_redo), "redo"),
            (ids!(menu_edit_cut), "cut"),
            (ids!(menu_edit_copy), "copy"),
            (ids!(menu_edit_paste), "paste"),
            (ids!(menu_edit_duplicate), "duplicate"),
            (ids!(menu_edit_delete), "delete"),
            (ids!(menu_edit_select_all), "select_all"),
            (ids!(menu_view_zoom_in), "zoom_in"),
            (ids!(menu_view_zoom_out), "zoom_out"),
            (ids!(menu_view_fit), "fit"),
            (ids!(menu_view_resources), "resources"),
            (ids!(menu_view_layers), "layers"),
            (ids!(menu_language_english), "english"),
            (ids!(menu_language_russian), "russian"),
            (ids!(menu_help_about), "about"),
            (ids!(menu_help_shortcuts), "shortcuts"),
            (ids!(menu_settings), "settings"),
        ] {
            self.ui
                .button(cx, id)
                .set_text(cx, desktop::text(key, language));
        }
        self.ui.button(cx, ids!(menu_window_next)).set_text(
            cx,
            if language == Language::Russian {
                "Следующая вкладка"
            } else {
                "Next Tab"
            },
        );
        self.ui.button(cx, ids!(menu_window_previous)).set_text(
            cx,
            if language == Language::Russian {
                "Предыдущая вкладка"
            } else {
                "Previous Tab"
            },
        );
        self.ui
            .button(cx, ids!(menu_window_close))
            .set_text(cx, desktop::text("close", language));
        self.ui.button(cx, ids!(menu_window_close_others)).set_text(
            cx,
            if language == Language::Russian {
                "Закрыть остальные"
            } else {
                "Close Other Tabs"
            },
        );
        self.ui.button(cx, ids!(menu_window_close_all)).set_text(
            cx,
            if language == Language::Russian {
                "Закрыть все"
            } else {
                "Close All Tabs"
            },
        );
        self.sync_document_tabs(cx);

        self.ui
            .label(cx, ids!(settings_title))
            .set_text(cx, desktop::text("settings", language));
        self.sync_settings_category(cx);
        self.ui.label(cx, ids!(settings_style_profile)).set_text(
            cx,
            &if language == Language::Russian {
                format!("Стабильный интерфейс: {profile_summary}")
            } else {
                format!("Production profile: {profile_summary}")
            },
        );
        self.ui.button(cx, ids!(settings_restore)).set_text(
            cx,
            if language == Language::Russian {
                if self.settings.restore_session {
                    "Восстанавливать сессию: Да"
                } else {
                    "Восстанавливать сессию: Нет"
                }
            } else if self.settings.restore_session {
                "Restore session: On"
            } else {
                "Restore session: Off"
            },
        );
        self.ui.button(cx, ids!(settings_close)).set_text(
            cx,
            if language == Language::Russian {
                "Закрыть"
            } else {
                "Close"
            },
        );
        self.sync_settings_controls(cx);
        self.ui
            .label(cx, ids!(about_title))
            .set_text(cx, desktop::text("about", language));
        let about_body = if language == Language::Russian {
            format!("GLYF Editor\nФормат GLYF v1\nОболочка приложения\nСтиль интерфейса: {profile_summary}")
        } else {
            format!("GLYF Editor\nGLYF format v1\nDesktop shell\nProduction UI: {profile_summary}")
        };
        self.ui
            .label(cx, ids!(about_body))
            .set_text(cx, &about_body);
        self.ui.button(cx, ids!(about_close)).set_text(
            cx,
            if language == Language::Russian {
                "Закрыть"
            } else {
                "Close"
            },
        );
        self.ui
            .label(cx, ids!(shortcuts_title))
            .set_text(cx, desktop::text("shortcuts", language));
        self.ui
            .label(cx, ids!(shortcuts_body))
            .set_text(cx, if language == Language::Russian {
                "Ctrl+N Создать · Ctrl+O Открыть · Ctrl+S Сохранить\nCtrl+Shift+S Сохранить как · Ctrl+W Закрыть вкладку\nCtrl+Z/Y Отменить/Повторить · Ctrl+C/V Копировать/Вставить\nCtrl+Tab Переключить вкладку · Alt+F4 Выход"
            } else {
                "Ctrl+N New · Ctrl+O Open · Ctrl+S Save\nCtrl+Shift+S Save As · Ctrl+W Close tab\nCtrl+Z/Y Undo/Redo · Ctrl+C/V Copy/Paste\nCtrl+Tab Switch tab · Alt+F4 Exit"
            });
        self.ui.button(cx, ids!(shortcuts_close)).set_text(
            cx,
            if language == Language::Russian {
                "Закрыть"
            } else {
                "Close"
            },
        );
        self.ui.label(cx, ids!(close_confirm_title)).set_text(
            cx,
            if language == Language::Russian {
                "Несохранённые изменения"
            } else {
                "Unsaved changes"
            },
        );
        self.ui.label(cx, ids!(close_confirm_body)).set_text(
            cx,
            if language == Language::Russian {
                "Сохранить изменения перед закрытием вкладки?"
            } else {
                "Save changes before closing this tab?"
            },
        );
        self.ui.button(cx, ids!(close_confirm_cancel)).set_text(
            cx,
            if language == Language::Russian {
                "Отмена"
            } else {
                "Cancel"
            },
        );
        self.ui.button(cx, ids!(close_confirm_dont_save)).set_text(
            cx,
            if language == Language::Russian {
                "Не сохранять"
            } else {
                "Don't Save"
            },
        );
        self.ui.button(cx, ids!(close_confirm_save)).set_text(
            cx,
            if language == Language::Russian {
                "Сохранить"
            } else {
                "Save"
            },
        );

        let recent_entries = desktop::recent_menu_entries(&self.settings.recent_files);
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
            let visible = index < recent_entries.len();
            self.ui.widget(cx, id).set_visible(cx, visible);
            if let Some(entry) = recent_entries.get(index) {
                self.ui.button(cx, id).set_text(cx, &entry.label);
            }
        }
        self.ui
            .label(cx, ids!(recent_empty))
            .set_text(cx, desktop::text("file.no_recent", language));
        self.ui
            .widget(cx, ids!(recent_empty))
            .set_visible(cx, recent_entries.is_empty());
        self.ui
            .button(cx, ids!(recent_clear))
            .set_text(cx, desktop::text("file.clear_recent", language));
        self.ui
            .widget(cx, ids!(recent_clear))
            .set_visible(cx, !recent_entries.is_empty());
        self.ui
            .widget(cx, ids!(recent_separator))
            .set_visible(cx, !recent_entries.is_empty());
        let mut active_shell_panel = None;
        for id in [
            ids!(file_menu),
            ids!(file_recent_submenu),
            ids!(edit_menu),
            ids!(view_menu),
            ids!(window_menu),
            ids!(language_menu),
            ids!(help_menu),
            ids!(settings_panel),
            ids!(about_panel),
            ids!(shortcuts_panel),
            ids!(close_confirm_panel),
            ids!(context_menu),
        ] {
            self.ui.widget(cx, id).set_visible(cx, false);
        }
        if self.context_menu_open {
            active_shell_panel = Some(ids!(context_menu));
        } else if let Some(menu) = self.menu_open.as_deref() {
            active_shell_panel = Some(match menu {
                "file" => ids!(file_menu),
                "edit" => ids!(edit_menu),
                "view" => ids!(view_menu),
                "window" => ids!(window_menu),
                "language" => ids!(language_menu),
                "help" => ids!(help_menu),
                "settings" => ids!(settings_panel),
                "about" => ids!(about_panel),
                "shortcuts" => ids!(shortcuts_panel),
                "close_confirm" => ids!(close_confirm_panel),
                _ => ids!(file_menu),
            });
        }
        if let Some(id) = active_shell_panel {
            self.ui.widget(cx, id).set_visible(cx, true);
            let key = if self.context_menu_open {
                "context".to_owned()
            } else {
                self.menu_open.clone().unwrap_or_else(|| "file".into())
            };
            let pos = self
                .overlay_pos
                .unwrap_or_else(|| self.shell_overlay_pos(cx, &key));
            let show_recent_submenu = key == "file" && self.file_recent_submenu_open();
            if key == "file" {
                self.position_shell_panel(cx, ids!(file_menu), pos);
                if show_recent_submenu {
                    let submenu_pos = self.file_recent_submenu_pos(cx, pos);
                    self.ui
                        .widget(cx, ids!(file_recent_submenu))
                        .set_visible(cx, true);
                    self.position_shell_panel(cx, ids!(file_recent_submenu), submenu_pos);
                }
            }
            if let Some(mut overlay) = self
                .ui
                .widget(cx, ids!(shell_overlay))
                .borrow_mut::<OverlayHost>()
            {
                // App owns outside-click routing so a click on another menu
                // title can switch menus in one gesture without the overlay
                // consuming that same pointer event first.
                if key == "file" {
                    overlay.show_chain(cx, false);
                } else {
                    overlay.show(cx, pos, self.shell_overlay_size(&key), false);
                }
            }
        } else if let Some(mut overlay) = self
            .ui
            .widget(cx, ids!(shell_overlay))
            .borrow_mut::<OverlayHost>()
        {
            overlay.hide(cx);
        }
        let sidebar_visible = self.panel_resources || self.panel_layers;
        self.ui
            .widget(cx, ids!(sidebar))
            .set_visible(cx, sidebar_visible);
        self.ui
            .widget(cx, ids!(resources_panel))
            .set_visible(cx, self.panel_resources);
        self.ui
            .widget(cx, ids!(layers_panel))
            .set_visible(cx, self.panel_layers);
        self.sync_menu_enabled(cx);
    }

    pub(crate) fn menu_state(&self) -> MenuState {
        let dirty_document_count = self.tabs.iter().enumerate().filter(|(index, tab)| {
            if *index == self.active_tab { self.dirty } else { tab.dirty }
        }).count();
        MenuState::from_snapshot(&MenuStateSnapshot {
            undo_len: self.undo_stack.len(),
            redo_len: self.redo_stack.len(),
            has_selection: self.selected_annotation.is_some_and(|index| index < self.annotations.len())
                || self.selected_part.is_some_and(|index| index < self.current_parts().len()),
            has_selectable_content: !self.annotations.is_empty() || !self.current_parts().is_empty(),
            has_clipboard: self.clipboard.is_some(),
            document_count: self.tabs.len(),
            active_document_dirty: self.dirty,
            active_document_has_path: !self.project_path.is_empty(),
            dirty_document_count,
        })
    }

    pub(crate) fn menu_command_enabled(&self, id: &str) -> Option<bool> {
        let state = self.menu_state();
        Some(match id {
            "menu.edit.undo" => state.undo,
            "menu.edit.redo" => state.redo,
            "menu.edit.cut" | "context.cut" | "clipboard.cut" => state.cut,
            "menu.edit.copy" | "context.copy" | "clipboard.copy" => state.copy,
            "menu.edit.paste" | "context.paste" | "clipboard.paste" => state.paste,
            "menu.edit.duplicate" | "context.duplicate" | "clipboard.duplicate" => state.duplicate,
            "menu.edit.delete" | "context.delete" => state.delete,
            "menu.edit.select_all" => state.select_all,
            "menu.file.save" | "menu.file.save_pack" => state.save,
            "menu.file.save_as" => state.save_as,
            "menu.file.close" | "menu.window.close" => state.close,
            "menu.window.close_others" => state.close_others,
            "menu.window.close_all" => state.close_all,
            "menu.window.next" => state.next_document,
            "menu.window.previous" => state.previous_document,
            _ => return None,
        })
    }

    pub(crate) fn sync_menu_enabled(&mut self, cx: &mut Cx) {
        let state = self.menu_state();
        let palette = self.style_profile.tokens(StyleTuning::default());
        for (id, enabled) in [
            (ids!(menu_edit_undo), state.undo), (ids!(menu_edit_redo), state.redo),
            (ids!(menu_edit_cut), state.cut), (ids!(menu_edit_copy), state.copy),
            (ids!(menu_edit_paste), state.paste), (ids!(menu_edit_duplicate), state.duplicate),
            (ids!(menu_edit_delete), state.delete), (ids!(menu_edit_select_all), state.select_all),
            (ids!(context_cut), state.cut), (ids!(context_copy), state.copy),
            (ids!(context_paste), state.paste), (ids!(context_duplicate), state.duplicate),
            (ids!(context_delete), state.delete),
            (ids!(menu_file_save), state.save), (ids!(menu_file_save_as), state.save_as),
            (ids!(menu_file_save_pack), state.save), (ids!(menu_file_close), state.close),
            (ids!(menu_window_close), state.close),
            (ids!(menu_window_close_others), state.close_others),
            (ids!(menu_window_close_all), state.close_all),
            (ids!(menu_window_next), state.next_document),
            (ids!(menu_window_previous), state.previous_document),
        ] {
            let mut button = self.ui.button(cx, id);
            button.set_enabled(cx, enabled);
            let ink = if enabled { palette.text_primary } else { palette.text_disabled };
            script_apply_eval!(cx, button, { draw_text +: { color: #(ink) } });
        }
    }

    pub(crate) fn toggle_sidebar(&mut self, cx: &mut Cx, panel: &str) {
        match panel {
            "resources" => {
                let open = !self.panel_resources;
                self.panel_resources = open;
                self.panel_layers = false;
            }
            "layers" => {
                let open = !self.panel_layers;
                self.panel_layers = open;
                self.panel_resources = false;
            }
            _ => return,
        }
        self.settings.panel_resources = self.panel_resources;
        self.settings.panel_layers = self.panel_layers;
        if let Err(error) = self.settings.save() {
            self.set_status(cx, format!("Settings save failed: {error}"));
        }
        self.sync_shell_labels(cx);
        self.ui.widget(cx, ids!(main_layout)).redraw(cx);
        #[cfg(all(windows, feature = "toolkit-uia"))]
        self.refresh_accessibility(cx);
    }

    pub(crate) fn shell_overlay_size(&self, key: &str) -> DVec2 {
        match key {
            // Explicit host bounds include the final Close row; OverlayHost intentionally clips to this size.
            "settings" => dvec2(760.0, 510.0),
            "about" => dvec2(340.0, 180.0),
            "shortcuts" => dvec2(430.0, 220.0),
            "close_confirm" => dvec2(380.0, 190.0),
            "context" => dvec2(260.0, 250.0),
            "file" => dvec2(360.0, 380.0),
            "edit" => dvec2(320.0, 370.0),
            "view" => dvec2(290.0, 230.0),
            "window" => dvec2(300.0, 250.0),
            "language" => dvec2(220.0, 110.0),
            "help" => dvec2(260.0, 110.0),
            _ => dvec2(300.0, 180.0),
        }
    }

    pub(crate) fn shell_overlay_pos(&self, cx: &Cx, key: &str) -> DVec2 {
        let viewport = self.ui.view(cx, ids!(preview)).area().rect(cx);
        let size = self.shell_overlay_size(key);
        let min_x = viewport.pos.x + 8.0;
        let min_y = viewport.pos.y + 8.0;
        dvec2(
            (viewport.pos.x + (viewport.size.x - size.x) * 0.5).clamp(
                min_x,
                (viewport.pos.x + viewport.size.x - size.x - 8.0).max(min_x),
            ),
            (viewport.pos.y + (viewport.size.y - size.y) * 0.5).clamp(
                min_y,
                (viewport.pos.y + viewport.size.y - size.y - 8.0).max(min_y),
            ),
        )
    }

    pub(crate) fn menu_anchor_pos(&self, cx: &Cx, menu: &str) -> Option<DVec2> {
        let rect = match menu {
            "file" => self.ui.button(cx, ids!(menu_file)).area().rect(cx),
            "edit" => self.ui.button(cx, ids!(menu_edit)).area().rect(cx),
            "view" => self.ui.button(cx, ids!(menu_view)).area().rect(cx),
            "window" => self.ui.button(cx, ids!(menu_window)).area().rect(cx),
            "language" => self.ui.button(cx, ids!(menu_language)).area().rect(cx),
            "help" => self.ui.button(cx, ids!(menu_help)).area().rect(cx),
            _ => return None,
        };
        if rect.size.x > 0.0 && rect.size.y > 0.0 {
            Some(dvec2(rect.pos.x, rect.pos.y + rect.size.y + 4.0))
        } else {
            None
        }
    }

    pub(crate) fn open_shell_panel(&mut self, cx: &mut Cx, menu: &str) {
        self.context_menu_open = false;
        self.menu_chain.close();
        self.menu_open = Some(menu.to_owned());
        self.overlay_pos = Some(self.shell_overlay_pos(cx, menu));
        self.sync_shell_labels(cx);
    }

    pub(crate) fn toggle_menu(&mut self, cx: &mut Cx, menu: &str) {
        let Some(menu_kind) = DesktopMenu::from_key(menu) else {
            self.open_shell_panel(cx, menu);
            return;
        };
        self.context_menu_open = false;
        self.menu_chain.toggle_root(menu_kind);
        self.menu_open = self.menu_chain.root().map(|menu| menu.key().to_owned());
        self.overlay_pos = self.menu_open.as_deref().and_then(|open| {
            self.menu_anchor_pos(cx, open)
                .or_else(|| Some(self.shell_overlay_pos(cx, open)))
        });
        self.sync_shell_labels(cx);
    }

    pub(crate) fn close_menu(&mut self, cx: &mut Cx) {
        let closing_confirmation = self.menu_open.as_deref() == Some("close_confirm");
        self.menu_open = None;
        self.menu_chain.close();
        self.context_menu_open = false;
        self.overlay_pos = None;
        if closing_confirmation {
            self.pending_tab_close = None;
            self.close_confirm_armed = false;
        }
        self.sync_shell_labels(cx);
        self.ui.widget(cx, ids!(main_layout)).redraw(cx);
        // Outside-pointer routing returns before the next Draw event. Publish
        // the logical visibility change now so semantic UI automation observes
        // the same closure that the user sees immediately.
        #[cfg(all(windows, feature = "toolkit-uia"))]
        self.refresh_accessibility(cx);
    }

    /// Close a shell popup before a new outside pointer action reaches Canvas.
    ///
    /// `DrawList2d` portal backgrounds are not consistently hit-testable on the
    /// current Makepad Windows path. The application therefore compares the
    /// pointer with the same explicit anchor and size that position the active
    /// OverlayHost, without relying on an ephemeral widget area.
    pub(crate) fn dismiss_shell_overlay_on_outside_pointer(&mut self, cx: &mut Cx, event: &Event) -> bool {
        if self.menu_open.is_none() && !self.context_menu_open {
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
        let key = if self.context_menu_open {
            "context"
        } else {
            self.menu_open.as_deref().unwrap_or("file")
        };
        let popup_pos = self
            .overlay_pos
            .unwrap_or_else(|| self.shell_overlay_pos(cx, key));
        let root_popup = Rect {
            pos: popup_pos,
            size: self.shell_overlay_size(key),
        };
        let contains_recent_submenu = key == "file" && self.file_recent_submenu_open() && {
            let submenu_pos = self.file_recent_submenu_pos(cx, popup_pos);
            Rect {
                pos: submenu_pos,
                size: self.file_recent_submenu_size(),
            }
            .contains(pointer_abs)
        };
        let contains_popup = root_popup.contains(pointer_abs) || contains_recent_submenu;
        if !contains_popup {
            let clicked_menu_title = [
                ids!(menu_file),
                ids!(menu_edit),
                ids!(menu_view),
                ids!(menu_window),
                ids!(menu_language),
                ids!(menu_help),
            ]
            .into_iter()
            .any(|id| self.ui.button(cx, id).area().rect(cx).contains(pointer_abs));
            if clicked_menu_title {
                // Let the target menu button handle this same event and replace
                // the active menu. This preserves the normal desktop menu-bar
                // interaction instead of requiring a close click first.
                return false;
            }
            self.close_menu(cx);
            self.set_status(cx, "Overlay dismissed");
            return true;
        }
        false
    }

    /// Keep a pending annotation modal without letting a pointer capture leak
    /// from the popup into Canvas. The comment field receives focus directly;
    /// button events still travel through their normal widget handlers.
    pub(crate) fn open_recent(&mut self, cx: &mut Cx) {
        let Some(path) = self.settings.recent_files.first().cloned() else {
            self.set_status(cx, "No recent files");
            return;
        };
        self.open_recent_path(cx, &path);
    }
    pub(crate) fn has_pending_work(&self) -> bool {
        self.pending_annotation.is_some()
            || self.pending_clip.is_some()
            || self.pending_part_popup.is_some()
    }

    pub(crate) fn active_overlay_kind(&self) -> &'static str {
        if let Some(annotation) = self.pending_annotation_document() {
            return if matches!(&annotation.geometry, GlyfAnnotationGeometry::Region { .. }) {
                "region_comment"
            } else {
                "point_comment"
            };
        }
        if self.pending_clip.is_some() {
            return "clip_popup";
        }
        if self.pending_part_popup.is_some() {
            return "part_popup";
        }
        if self.context_menu_open {
            return "context_menu";
        }
        match self.menu_open.as_deref() {
            Some("file") => "file_menu",
            Some("edit") => "edit_menu",
            Some("view") => "view_menu",
            Some("window") => "window_menu",
            Some("language") => "language_menu",
            Some("help") => "help_menu",
            Some("settings") => "settings",
            Some("shortcuts") => "shortcuts",
            Some("about") => "about",
            Some("close_confirm") => "close_confirm",
            Some(_) => "menu",
            None => "none",
        }
    }

    pub(crate) fn set_status(&mut self, cx: &mut Cx, message: impl Into<String>) {
        self.status_text = message.into();
        self.ui
            .label(cx, ids!(status))
            .set_text(cx, &desktop::localized_status(&self.status_text, self.settings.language));
    }
}
