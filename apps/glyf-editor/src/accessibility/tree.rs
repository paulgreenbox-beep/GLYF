//! Semantic tree projection; not a second automation/transport implementation.

use super::*;

impl App {
    #[cfg(windows)]
    pub(crate) fn shell_accessibility_visible(&self, id: &str) -> bool {
        if let Some(raw) = id.strip_prefix("recent.item.") {
            return raw.parse::<usize>().is_ok_and(|index| {
                self.file_recent_submenu_open()
                    && index < desktop::recent_menu_entries(&self.settings.recent_files).len()
                    && index < RECENT_MENU_CAPACITY
            });
        }
        if matches!(
            id,
            "menu.file.open_recent.submenu" | "recent.empty" | "recent.clear"
        ) {
            return self.file_recent_submenu_open();
        }
        match id {
            "file_menu"
            | "menu.file.new"
            | "menu.file.open"
            | "menu.file.recent"
            | "menu.file.save"
            | "menu.file.save_as"
            | "menu.file.save_pack"
            | "menu.file.close"
            | "menu.file.exit" => self.menu_open.as_deref() == Some("file"),
            "edit_menu"
            | "menu.edit.undo"
            | "menu.edit.redo"
            | "menu.edit.cut"
            | "menu.edit.copy"
            | "menu.edit.paste"
            | "menu.edit.duplicate"
            | "menu.edit.delete"
            | "menu.edit.select_all"
            | "menu.settings" => self.menu_open.as_deref() == Some("edit"),
            "view_menu"
            | "menu.view.zoom_in"
            | "menu.view.zoom_out"
            | "menu.view.fit"
            | "menu.view.resources"
            | "menu.view.layers" => self.menu_open.as_deref() == Some("view"),
            "window_menu"
            | "menu.window.next"
            | "menu.window.previous"
            | "menu.window.close"
            | "menu.window.close_others"
            | "menu.window.close_all" => self.menu_open.as_deref() == Some("window"),
            "language_menu" | "menu.language.english" | "menu.language.russian" => {
                self.menu_open.as_deref() == Some("language")
            }
            "help_menu" | "menu.help.about" | "menu.help.shortcuts" => {
                self.menu_open.as_deref() == Some("help")
            }
            "settings_panel" | "settings.restore" | "settings.close" => {
                self.menu_open.as_deref() == Some("settings")
            }
            "about_panel" | "about.close" => self.menu_open.as_deref() == Some("about"),
            "shortcuts_panel" | "shortcuts.close" => self.menu_open.as_deref() == Some("shortcuts"),
            "close_confirm_panel"
            | "tab.close.confirm.cancel"
            | "tab.close.confirm.dont_save"
            | "tab.close.confirm.save" => self.menu_open.as_deref() == Some("close_confirm"),
            "context_menu" | "context.copy" | "context.cut" | "context.paste"
            | "context.duplicate" | "context.delete" => self.context_menu_open,
            "resources_panel" => self.panel_resources,
            "layers_panel" => self.panel_layers,
            _ => true,
        }
    }

    #[cfg(windows)]
    pub(crate) fn accessibility_tree(
        &self,
        cx: &Cx,
        hwnd: HWND,
        generation: u64,
    ) -> Result<AccessibilityTree, String> {
        let screen = ScreenTransform::for_window(hwnd)?;
        let mut window_rect = RECT::default();
        unsafe { GetWindowRect(hwnd, &mut window_rect) }.map_err(|error| error.to_string())?;
        let mut root = AccessibilityNode::new(
            "window.main",
            AccessibilityRole::Window,
            WINDOW_TITLE,
            AccessibilityRect {
                x: window_rect.left as f64,
                y: window_rect.top as f64,
                width: (window_rect.right - window_rect.left).max(0) as f64,
                height: (window_rect.bottom - window_rect.top).max(0) as f64,
            },
        );
        let mut toolbar = AccessibilityNode::new(
            "toolbar.main",
            AccessibilityRole::ToolBar,
            "Project actions",
            screen.rect(self.ui.view(cx, ids!(toolbar)).area().rect(cx)),
        );
        let mut image_path = AccessibilityNode::new(
            "image.path",
            AccessibilityRole::Edit,
            "Image path",
            screen.rect(
                self.ui
                    .text_input(cx, ids!(image_path_input))
                    .area()
                    .rect(cx),
            ),
        );
        image_path.actions = AccessibilityActions {
            focus: true,
            set_value: true,
            ..Default::default()
        };
        image_path.value = Some(self.ui.text_input(cx, ids!(image_path_input)).text());
        let mut image_name = AccessibilityNode::new(
            "image.name",
            AccessibilityRole::Edit,
            "Image name",
            screen.rect(self.ui.text_input(cx, ids!(name_input)).area().rect(cx)),
        );
        image_name.value = Some(self.ui.text_input(cx, ids!(name_input)).text());
        image_name.actions = AccessibilityActions {
            focus: true,
            set_value: true,
            ..Default::default()
        };
        let mut image_comment = AccessibilityNode::new(
            "image.comment",
            AccessibilityRole::Edit,
            "Image comment",
            screen.rect(self.ui.text_input(cx, ids!(comment_input)).area().rect(cx)),
        );
        image_comment.value = Some(self.ui.text_input(cx, ids!(comment_input)).text());
        image_comment.actions = AccessibilityActions {
            focus: true,
            set_value: true,
            ..Default::default()
        };
        toolbar.children.push(image_name);
        toolbar.children.push(image_comment);
        toolbar.children.push(image_path);
        for (id, name, widget_id) in [
            ("project.new", "New project", ids!(new_project)),
            ("project.save", "Save project", ids!(save_project)),
            ("sidebar.images", "Show images", ids!(sidebar_images)),
            ("sidebar.layers", "Show layers", ids!(sidebar_layers)),
            ("image.add", "Add image", ids!(add_image)),
            ("image.remove", "Remove image", ids!(remove_image)),
            ("image.update", "Update image", ids!(update_image)),
        ] {
            let mut node = AccessibilityNode::new(
                id,
                AccessibilityRole::Button,
                name,
                screen.rect(self.ui.button(cx, widget_id).area().rect(cx)),
            );
            node.actions.invoke = true;
            toolbar.children.push(node);
        }
        for (id, name, widget_id) in [
            ("tool.select", "Select canvas tool", ids!(tool_select)),
            ("tool.point", "Point canvas tool", ids!(tool_point)),
            ("tool.region", "Region canvas tool", ids!(tool_region)),
            ("annotation.edit", "Edit annotation", ids!(annotation_edit)),
            (
                "annotation.delete",
                "Delete annotation",
                ids!(annotation_delete),
            ),
            ("part.add", "Add part", ids!(part_add)),
            ("part.add_clip", "Add clipped part", ids!(part_add_clip)),
            ("part.remove", "Delete part", ids!(part_remove)),
            ("part.up", "Move part up", ids!(part_up)),
            ("part.down", "Move part down", ids!(part_down)),
            ("part.back", "Send part to back", ids!(part_back)),
            ("part.front", "Bring part to front", ids!(part_front)),
        ] {
            let mut node = AccessibilityNode::new(
                id,
                AccessibilityRole::Button,
                name,
                screen.rect(self.ui.button(cx, widget_id).area().rect(cx)),
            );
            node.actions.invoke = true;
            toolbar.children.push(node);
        }
        let mut status = AccessibilityNode::new(
            "status",
            AccessibilityRole::Text,
            "Status",
            screen.rect(self.ui.label(cx, ids!(status)).area().rect(cx)),
        );
        status.value = Some(desktop::localized_status(&self.status_text, self.settings.language));
        let mut image_list = AccessibilityNode::new(
            "image.list",
            AccessibilityRole::List,
            "Images",
            screen.rect(self.ui.label(cx, ids!(images_list)).area().rect(cx)),
        );
        image_list.value = Some(format!("{} image(s)", self.images.len()));
        for (index, image) in self.images.iter().enumerate() {
            let mut item = AccessibilityNode::new(
                format!("image.item.{index}"),
                AccessibilityRole::ListItem,
                image.name.clone(),
                screen.rect(self.ui.label(cx, ids!(images_list)).area().rect(cx)),
            );
            item.value = Some(format!(
                "role: {}\nsource: {}\ncomment: {}",
                image.name, image.source, image.comment
            ));
            item.selected = Some(Some(index) == self.selected);
            item.actions.invoke = true;
            image_list.children.push(item);
        }
        let mut image_count = AccessibilityNode::new(
            "image.count",
            AccessibilityRole::Text,
            "Image count",
            screen.rect(self.ui.label(cx, ids!(images_list)).area().rect(cx)),
        );
        image_count.value = Some(self.images.len().to_string());
        let preview = AccessibilityNode::new(
            "preview",
            AccessibilityRole::Pane,
            "Image preview",
            screen.rect(self.ui.view(cx, ids!(preview)).area().rect(cx)),
        );

        let mut part_list = AccessibilityNode::new(
            "part.list",
            AccessibilityRole::List,
            "Layers",
            screen.rect(self.ui.label(cx, ids!(layers_list)).area().rect(cx)),
        );
        let parts = self.current_parts().to_vec();
        part_list.value = Some(format!("{} part(s)", parts.len()));
        for (index, part) in parts.iter().enumerate() {
            let mut item = AccessibilityNode::new(
                format!("part.item.{index}"),
                AccessibilityRole::ListItem,
                part.name.clone(),
                screen.rect(self.ui.label(cx, ids!(layers_list)).area().rect(cx)),
            );
            item.value = Some(format!(
                "source: {}\nclip: {}\nrel1: {:.4} {:.4}\nrel2: {:.4} {:.4}\nparent: {}\ncomment: {}",
                part.source,
                part.clip
                    .map(|clip| format!("{:.0} {:.0} {:.0} {:.0}", clip.x, clip.y, clip.w, clip.h))
                    .unwrap_or_else(|| "full image".into()),
                part.rel1.x,
                part.rel1.y,
                part.rel2.x,
                part.rel2.y,
                part.parent.as_deref().unwrap_or("<none>"),
                part.comment
            ));
            item.selected = Some(Some(index) == self.selected_part);
            item.actions.invoke = true;
            part_list.children.push(item);
        }
        let mut part_count = AccessibilityNode::new(
            "part.count",
            AccessibilityRole::Text,
            "Part count",
            screen.rect(self.ui.label(cx, ids!(layers_list)).area().rect(cx)),
        );
        part_count.value = Some(parts.len().to_string());

        let annotation_list = {
            let mut list = AccessibilityNode::new(
                "annotation.list",
                AccessibilityRole::List,
                "Canvas annotations",
                screen.rect(self.ui.label(cx, ids!(annotations_list)).area().rect(cx)),
            );
            list.value = Some(format!("{} annotation(s)", self.annotations.len()));
            for (index, annotation) in self.annotations.iter().enumerate() {
                let mut item = AccessibilityNode::new(
                    format!("annotation.item.{index}"),
                    AccessibilityRole::ListItem,
                    annotation.name.clone(),
                    screen.rect(self.ui.label(cx, ids!(annotations_list)).area().rect(cx)),
                );
                item.value = Some(format!(
                    "role: {}\ngeometry: {}\n{}\ncomment: {}",
                    annotation.role,
                    annotation_geometry_summary(annotation),
                    annotation_coordinates_summary(annotation),
                    annotation.comment
                ));
                item.selected = Some(Some(index) == self.selected_annotation);
                item.actions.invoke = true;
                list.children.push(item);
            }
            list
        };
        let annotation_bounds =
            screen.rect(self.ui.label(cx, ids!(annotations_list)).area().rect(cx));
        let annotation_count = accessibility_text_node(
            "annotation.count",
            "Annotation count",
            annotation_bounds,
            self.annotations.len().to_string(),
        );
        let annotation_selected = accessibility_text_node(
            "annotation.selected",
            "Selected annotation",
            annotation_bounds,
            self.selected_annotation
                .map(|index| index.to_string())
                .unwrap_or_else(|| "-1".into()),
        );
        let selected_annotation = self
            .selected_annotation
            .and_then(|index| self.annotations.get(index));
        let mut annotation_role = AccessibilityNode::new(
            "annotation.role",
            AccessibilityRole::Edit,
            "Annotation role",
            annotation_bounds,
        );
        annotation_role.value = Some(
            selected_annotation
                .map(|annotation| annotation.role.clone())
                .unwrap_or_default(),
        );
        annotation_role.actions = AccessibilityActions {
            focus: true,
            set_value: true,
            ..Default::default()
        };
        let annotation_geometry = accessibility_text_node(
            "annotation.geometry",
            "Annotation geometry",
            annotation_bounds,
            selected_annotation
                .map(annotation_geometry_summary)
                .unwrap_or_else(|| "".into()),
        );
        let annotation_comment_summary = accessibility_text_node(
            "annotation.comment",
            "Annotation comment",
            annotation_bounds,
            selected_annotation
                .map(|annotation| annotation.comment.clone())
                .unwrap_or_else(|| "".into()),
        );
        let region_drag_active = accessibility_text_node(
            "region.drag.active",
            "Region drag active",
            annotation_bounds,
            self.canvas_interaction_active(cx).to_string(),
        );
        let region_selection = accessibility_text_node(
            "region.selection",
            "Region selection",
            annotation_bounds,
            selected_annotation
                .filter(|annotation| {
                    matches!(&annotation.geometry, GlyfAnnotationGeometry::Region { .. })
                })
                .map(|_| self.selected_annotation.unwrap().to_string())
                .unwrap_or_else(|| "-1".into()),
        );
        let region_pending = accessibility_text_node(
            "region.pending",
            "Pending Region",
            annotation_bounds,
            self.pending_annotation_document()
                .is_some_and(|annotation| {
                    matches!(&annotation.geometry, GlyfAnnotationGeometry::Region { .. })
                })
                .to_string(),
        );
        let region_hover = accessibility_text_node(
            "region.hover",
            "Hovered Region",
            annotation_bounds,
            self.hovered_annotation
                .and_then(|index| self.annotations.get(index))
                .is_some_and(|annotation| {
                    matches!(&annotation.geometry, GlyfAnnotationGeometry::Region { .. })
                })
                .to_string(),
        );
        let mut region_input = AccessibilityNode::new(
            "annotation.region.input",
            AccessibilityRole::Edit,
            "Region relative rectangle",
            annotation_bounds,
        );
        region_input.value = Some(
            selected_annotation
                .and_then(|annotation| match &annotation.geometry {
                    GlyfAnnotationGeometry::Region { rel1, rel2 } => Some(format!(
                        "{:.6},{:.6},{:.6},{:.6}",
                        rel1.x, rel1.y, rel2.x, rel2.y
                    )),
                    GlyfAnnotationGeometry::Point { .. } => None,
                })
                .unwrap_or_default(),
        );
        region_input.actions = AccessibilityActions {
            focus: true,
            set_value: true,
            ..Default::default()
        };
        let coordinate_nodes = annotation_coordinate_nodes(annotation_bounds, selected_annotation);

        let pending_annotation = self.pending_annotation_document();
        let popup_visible = pending_annotation.is_some();
        let popup_pos = pending_annotation
            .map(|annotation| self.annotation_popup_position(cx, annotation))
            .unwrap_or(DVec2 { x: 0.0, y: 0.0 });
        let popup_widget_rect = self.ui.view(cx, ids!(annotation_popup)).area().rect(cx);
        let popup_rect =
            if popup_visible && popup_widget_rect.size.x > 0.0 && popup_widget_rect.size.y > 0.0 {
                screen.rect(popup_widget_rect)
            } else if popup_visible {
                screen.rect(Rect {
                    pos: popup_pos,
                    size: dvec2(310.0, 128.0),
                })
            } else {
                screen.rect(self.ui.view(cx, ids!(preview)).area().rect(cx))
            };
        let mut annotation_popup = AccessibilityNode::new(
            "annotation.popup",
            AccessibilityRole::Pane,
            "Annotation comment popup",
            popup_rect,
        );
        annotation_popup.visible = popup_visible;
        annotation_popup.enabled = popup_visible;
        let comment_widget_rect = self
            .ui
            .text_input(cx, ids!(annotation_popup.comment_input))
            .area()
            .rect(cx);
        let comment_rect = if popup_visible
            && comment_widget_rect.size.x > 0.0
            && comment_widget_rect.size.y > 0.0
        {
            screen.rect(comment_widget_rect)
        } else {
            screen.rect(Rect {
                pos: dvec2(popup_pos.x + 10.0, popup_pos.y + 34.0),
                size: dvec2(290.0, 44.0),
            })
        };
        let mut annotation_comment = AccessibilityNode::new(
            "annotation.popup.comment",
            AccessibilityRole::Edit,
            "Annotation comment",
            comment_rect,
        );
        annotation_comment.value = Some(self.annotation_draft.clone());
        annotation_comment.focused = popup_visible
            && cx.has_key_focus(
                self.ui
                    .text_input(cx, ids!(annotation_popup.comment_input))
                    .area(),
            );
        annotation_comment.actions = AccessibilityActions {
            focus: popup_visible,
            set_value: popup_visible,
            ..Default::default()
        };
        annotation_comment.visible = popup_visible;
        annotation_comment.enabled = popup_visible;
        annotation_popup.children.push(annotation_comment.clone());
        for (id, name, widget_id, fallback_pos) in [
            (
                "annotation.popup.cancel",
                "Cancel annotation",
                ids!(annotation_popup.actions.cancel),
                dvec2(popup_pos.x + 166.0, popup_pos.y + 92.0),
            ),
            (
                "annotation.popup.commit",
                "Commit annotation",
                ids!(annotation_popup.actions.commit),
                dvec2(popup_pos.x + 246.0, popup_pos.y + 92.0),
            ),
            (
                "annotation.popup.done",
                "Done",
                ids!(annotation_popup.actions.commit),
                dvec2(popup_pos.x + 246.0, popup_pos.y + 92.0),
            ),
        ] {
            let widget_rect = self.ui.button(cx, widget_id).area().rect(cx);
            let button_rect =
                if popup_visible && widget_rect.size.x > 0.0 && widget_rect.size.y > 0.0 {
                    screen.rect(widget_rect)
                } else {
                    screen.rect(Rect {
                        pos: fallback_pos,
                        size: dvec2(64.0, 28.0),
                    })
                };
            let mut node = AccessibilityNode::new(id, AccessibilityRole::Button, name, button_rect);
            node.actions.invoke = popup_visible;
            node.visible = popup_visible;
            node.enabled = popup_visible;
            annotation_popup.children.push(node);
        }
        let canvas_viewport = self.canvas_rect(cx);
        let canvas_content = zoomed_content_rect(
            canvas_viewport,
            self.selected_image_size(),
            self.canvas_zoom_value(),
            self.canvas_pan,
        );
        let mut canvas = AccessibilityNode::new(
            "canvas.surface",
            AccessibilityRole::Pane,
            "Spatial canvas surface",
            screen.rect(canvas_viewport),
        );
        let mut canvas_content_node = AccessibilityNode::new(
            "canvas.content",
            AccessibilityRole::Pane,
            "Spatial canvas content",
            screen.rect(canvas_content),
        );
        for (index, part) in parts.iter().enumerate() {
            let pos = canvas_content.pos + relative_to_canvas(canvas_content.size, part.rel1);
            let end = canvas_content.pos + relative_to_canvas(canvas_content.size, part.rel2);
            let mut node = AccessibilityNode::new(
                format!("part.canvas.{index}"),
                AccessibilityRole::ListItem,
                format!("Canvas part {}", part.name),
                screen.rect(Rect {
                    pos,
                    size: dvec2((end.x - pos.x).max(1.0), (end.y - pos.y).max(1.0)),
                }),
            );
            node.value = Some(format!(
                "{:.6},{:.6},{:.6},{:.6}",
                part.rel1.x, part.rel1.y, part.rel2.x, part.rel2.y
            ));
            node.selected = Some(Some(index) == self.selected_part);
            node.actions.invoke = true;
            canvas_content_node.children.push(node);
        }
        for (index, annotation) in self.annotations.iter().enumerate() {
            let rect = match &annotation.geometry {
                GlyfAnnotationGeometry::Point { rel } => {
                    let center = canvas_content.pos + relative_to_canvas(canvas_content.size, *rel);
                    Rect {
                        pos: center - dvec2(10.0, 10.0),
                        size: dvec2(20.0, 20.0),
                    }
                }
                GlyfAnnotationGeometry::Region { rel1, rel2 } => {
                    let pos = canvas_content.pos + relative_to_canvas(canvas_content.size, *rel1);
                    let end = canvas_content.pos + relative_to_canvas(canvas_content.size, *rel2);
                    Rect {
                        pos,
                        size: dvec2((end.x - pos.x).max(1.0), (end.y - pos.y).max(1.0)),
                    }
                }
            };
            let mut node = AccessibilityNode::new(
                format!("annotation.canvas.{index}"),
                AccessibilityRole::ListItem,
                format!("Canvas annotation {}", annotation.name),
                screen.rect(rect),
            );
            node.value = Some(annotation_coordinates_summary(annotation));
            node.selected = Some(Some(index) == self.selected_annotation);
            node.actions.invoke = true;
            canvas_content_node.children.push(node);
        }
        canvas.children.push(canvas_content_node);
        let preview_bounds = screen.rect(self.ui.view(cx, ids!(preview)).area().rect(cx));
        let mut source_mode_node = AccessibilityNode::new(
            "source.mode",
            AccessibilityRole::Button,
            "Open Source mode",
            preview_bounds,
        );
        source_mode_node.value = Some(if self.source_mode {
            "true".into()
        } else {
            "false".into()
        });
        source_mode_node.actions.invoke = true;
        let source_image = accessibility_text_node(
            "source.image",
            "Source image",
            preview_bounds,
            self.selected_image_name(),
        );
        let source_size = self.selected_source_size();
        let source_width = accessibility_text_node(
            "source.width",
            "Source width",
            preview_bounds,
            source_size
                .map(|size| format!("{:.0}", size.x))
                .unwrap_or_default(),
        );
        let source_height = accessibility_text_node(
            "source.height",
            "Source height",
            preview_bounds,
            source_size
                .map(|size| format!("{:.0}", size.y))
                .unwrap_or_default(),
        );
        let editor_mode = accessibility_text_node(
            "editor.mode",
            "Editor mode",
            preview_bounds,
            if self.source_mode {
                format!("SOURCE: {}", self.selected_image_name())
            } else {
                "COMPOSITION".to_owned()
            },
        );
        let project_dirty = accessibility_text_node(
            "project.dirty",
            "Unsaved state",
            preview_bounds,
            self.dirty.to_string(),
        );
        let selected_part_node = accessibility_text_node(
            "part.selected",
            "Selected part",
            preview_bounds,
            self.selected_part
                .map(|index| index.to_string())
                .unwrap_or_else(|| "-1".into()),
        );
        let canvas_zoom = accessibility_text_node(
            "canvas.zoom",
            "Canvas zoom",
            preview_bounds,
            format!("{:.4}", self.canvas_zoom_value()),
        );
        let canvas_pan = accessibility_text_node(
            "canvas.pan",
            "Canvas pan",
            preview_bounds,
            format!("{:.2},{:.2}", self.canvas_pan.x, self.canvas_pan.y),
        );
        let mut canvas_fit = AccessibilityNode::new(
            "canvas.fit",
            AccessibilityRole::Button,
            "Fit composition canvas",
            screen.rect(self.ui.button(cx, ids!(canvas_fit)).area().rect(cx)),
        );
        canvas_fit.actions.invoke = true;
        let mut canvas_100 = AccessibilityNode::new(
            "canvas.100",
            AccessibilityRole::Button,
            "Composition canvas 100 percent",
            screen.rect(self.ui.button(cx, ids!(canvas_100)).area().rect(cx)),
        );
        canvas_100.actions.invoke = true;
        let mut canvas_zoom_out = AccessibilityNode::new(
            "canvas.zoom.out",
            AccessibilityRole::Button,
            "Zoom composition out",
            screen.rect(self.ui.button(cx, ids!(canvas_zoom_out)).area().rect(cx)),
        );
        canvas_zoom_out.actions.invoke = true;
        let mut canvas_zoom_in = AccessibilityNode::new(
            "canvas.zoom.in",
            AccessibilityRole::Button,
            "Zoom composition in",
            screen.rect(self.ui.button(cx, ids!(canvas_zoom_in)).area().rect(cx)),
        );
        canvas_zoom_in.actions.invoke = true;
        let source_zoom = accessibility_text_node(
            "source.zoom",
            "Source zoom",
            preview_bounds,
            format!("{:.4}", self.source_zoom_value()),
        );
        let source_pan = accessibility_text_node(
            "source.pan",
            "Source pan",
            preview_bounds,
            format!("{:.2},{:.2}", self.source_pan.x, self.source_pan.y),
        );
        let mut source_toolbar = AccessibilityNode::new(
            "source.toolbar",
            AccessibilityRole::ToolBar,
            "Source toolbar",
            screen.rect(self.ui.view(cx, ids!(source_toolbar)).area().rect(cx)),
        );
        for (id, label, widget_id) in [
            ("source.back", "Back to composition", ids!(source_back)),
            (
                "tool.source.select",
                "Source select tool",
                ids!(source_select),
            ),
            ("tool.source.clip", "Source clip tool", ids!(source_clip)),
            ("source.fit", "Fit source", ids!(source_fit)),
            ("source.100", "Source 100 percent", ids!(source_100)),
            ("source.zoom.out", "Zoom source out", ids!(source_zoom_out)),
            ("source.zoom.in", "Zoom source in", ids!(source_zoom_in)),
        ] {
            let mut button = AccessibilityNode::new(
                id,
                AccessibilityRole::Button,
                label,
                screen.rect(self.ui.button(cx, widget_id).area().rect(cx)),
            );
            button.actions.invoke = true;
            source_toolbar.children.push(button);
        }
        let selected_clip = self
            .pending_clip
            .as_ref()
            .map(|pending| pending.clip)
            .or(self.source_clip);
        let source_viewport = self.source_canvas_rect(cx);
        let source_content = zoomed_content_rect(
            source_viewport,
            source_size,
            self.source_zoom_value(),
            self.source_pan,
        );
        let mut source_surface = AccessibilityNode::new(
            "source.surface",
            AccessibilityRole::Pane,
            "Source image surface",
            screen.rect(source_viewport),
        );
        let mut source_content_node = AccessibilityNode::new(
            "source.content",
            AccessibilityRole::Pane,
            "Source image content",
            screen.rect(source_content),
        );
        if let (Some(size), Some(clip)) = (source_size, selected_clip) {
            let mut clip_node = AccessibilityNode::new(
                "clip.canvas",
                AccessibilityRole::ListItem,
                "Source clip on canvas",
                screen.rect(source_rect_to_screen(
                    source_viewport,
                    size,
                    self.source_zoom_value(),
                    self.source_pan,
                    clip,
                )),
            );
            clip_node.value = Some(format!(
                "{:.0},{:.0},{:.0},{:.0}",
                clip.x, clip.y, clip.w, clip.h
            ));
            clip_node.selected = Some(true);
            source_content_node.children.push(clip_node);
        }
        source_surface.children.push(source_content_node);
        for (id, value) in [
            (
                "clip.x",
                selected_clip
                    .map(|clip| format!("{:.0}", clip.x))
                    .unwrap_or_default(),
            ),
            (
                "clip.y",
                selected_clip
                    .map(|clip| format!("{:.0}", clip.y))
                    .unwrap_or_default(),
            ),
            (
                "clip.w",
                selected_clip
                    .map(|clip| format!("{:.0}", clip.w))
                    .unwrap_or_default(),
            ),
            (
                "clip.h",
                selected_clip
                    .map(|clip| format!("{:.0}", clip.h))
                    .unwrap_or_default(),
            ),
        ] {
            root.children
                .push(accessibility_text_node(id, id, preview_bounds, value));
        }
        let clip_selection = accessibility_text_node(
            "clip.selection",
            "Selected source clip",
            preview_bounds,
            selected_clip
                .map(|clip| format!("{:.0},{:.0},{:.0},{:.0}", clip.x, clip.y, clip.w, clip.h))
                .unwrap_or_default(),
        );
        let mut clip_input = AccessibilityNode::new(
            "clip.input",
            AccessibilityRole::Edit,
            "Source clip pixel rectangle",
            preview_bounds,
        );
        clip_input.value = Some(
            selected_clip
                .map(|clip| format!("{:.0},{:.0},{:.0},{:.0}", clip.x, clip.y, clip.w, clip.h))
                .unwrap_or_default(),
        );
        clip_input.actions = AccessibilityActions {
            focus: true,
            set_value: true,
            ..Default::default()
        };
        let clip_drag_active = accessibility_text_node(
            "clip.drag.active",
            "Source clip drag active",
            preview_bounds,
            self.ui
                .widget(cx, ids!(source_canvas))
                .borrow::<SourceCanvas>()
                .map(|canvas| canvas.drag_active())
                .unwrap_or(false)
                .to_string(),
        );
        let clip_popup_bounds = if self.pending_clip.is_some() {
            screen.rect(Rect {
                pos: self.source_popup_position(cx),
                size: dvec2(350.0, 190.0),
            })
        } else {
            preview_bounds
        };
        let mut clip_popup_node = AccessibilityNode::new(
            "clip.popup",
            AccessibilityRole::Pane,
            "Source clip popup",
            clip_popup_bounds,
        );
        for (id, label, value) in [
            (
                "clip.popup.name",
                "Part name",
                self.pending_clip
                    .as_ref()
                    .map(|pending| pending.draft_name.clone())
                    .unwrap_or_default(),
            ),
            (
                "clip.popup.comment",
                "Part comment",
                self.pending_clip
                    .as_ref()
                    .map(|pending| pending.draft_comment.clone())
                    .unwrap_or_default(),
            ),
        ] {
            let mut field =
                AccessibilityNode::new(id, AccessibilityRole::Edit, label, clip_popup_bounds);
            field.value = Some(value);
            field.actions = AccessibilityActions {
                focus: true,
                set_value: true,
                ..Default::default()
            };
            clip_popup_node.children.push(field);
        }
        for (id, label) in [
            ("clip.popup.create", "Create or update part from clip"),
            ("clip.popup.cancel", "Cancel source clip"),
        ] {
            let mut button =
                AccessibilityNode::new(id, AccessibilityRole::Button, label, clip_popup_bounds);
            button.actions.invoke = true;
            clip_popup_node.children.push(button);
        }

        let part_popup_bounds = self
            .pending_part_popup
            .as_ref()
            .map(|pending| {
                screen.rect(Rect {
                    pos: self.part_popup_position(cx, pending.index),
                    size: dvec2(350.0, 170.0),
                })
            })
            .unwrap_or(preview_bounds);
        let mut part_popup_node = AccessibilityNode::new(
            "part.popup",
            AccessibilityRole::Pane,
            "Inline part metadata popup",
            part_popup_bounds,
        );
        for (id, label, value) in [
            (
                "part.popup.name",
                "Part name",
                self.pending_part_popup
                    .as_ref()
                    .map(|pending| pending.draft_name.clone())
                    .unwrap_or_default(),
            ),
            (
                "part.popup.comment",
                "Part comment",
                self.pending_part_popup
                    .as_ref()
                    .map(|pending| pending.draft_comment.clone())
                    .unwrap_or_default(),
            ),
        ] {
            let mut field =
                AccessibilityNode::new(id, AccessibilityRole::Edit, label, part_popup_bounds);
            field.value = Some(value);
            field.actions = AccessibilityActions {
                focus: true,
                set_value: true,
                ..Default::default()
            };
            part_popup_node.children.push(field);
        }
        for (id, label) in [
            ("part.popup.commit", "Commit part metadata"),
            ("part.popup.cancel", "Cancel part metadata"),
        ] {
            let mut button =
                AccessibilityNode::new(id, AccessibilityRole::Button, label, part_popup_bounds);
            button.actions.invoke = true;
            part_popup_node.children.push(button);
        }

        let shell_bounds = screen.rect(self.ui.button(cx, ids!(menu_file)).area().rect(cx));
        let overlay_kind = self.active_overlay_kind();
        let overlay_active = overlay_kind != "none";
        let mut shell_nodes = Vec::new();
        shell_nodes.push(accessibility_text_node(
            "shell.style_profile",
            "Production style profile",
            shell_bounds,
            self.style_profile.summary(),
        ));
        for (id, label, visible) in [
            ("overlay.active", "Overlay active", true),
            ("overlay.kind", "Overlay kind", true),
            ("menu.overlay", "Menu overlay", self.menu_open.is_some()),
            (
                "context.menu",
                "Context menu overlay",
                self.context_menu_open,
            ),
            (
                "dialog.help",
                "Help overlay",
                matches!(self.menu_open.as_deref(), Some("about" | "shortcuts")),
            ),
            (
                "dialog.settings",
                "Settings overlay",
                self.menu_open.as_deref() == Some("settings"),
            ),
            (
                "dialog.close_confirm",
                "Close document confirmation",
                self.menu_open.as_deref() == Some("close_confirm"),
            ),
        ] {
            let value = match id {
                "overlay.active" => overlay_active.to_string(),
                "overlay.kind" => overlay_kind.to_owned(),
                _ => overlay_kind.to_owned(),
            };
            let mut node = accessibility_text_node(id, label, shell_bounds, value);
            node.visible = visible;
            node.enabled = visible;
            shell_nodes.push(node);
        }
        let mut recent_count = accessibility_text_node(
            "recent.count",
            "Recent file count",
            shell_bounds,
            self.settings.recent_files.len().to_string(),
        );
        recent_count.visible = self.file_recent_submenu_open();
        recent_count.enabled = false;
        shell_nodes.push(recent_count);
        let shell_control_bounds = |id: &str| {
            let rect = match id {
                "file_menu" => self.ui.widget(cx, ids!(file_menu)).area().rect(cx),
                "menu.file.new" => self.ui.button(cx, ids!(menu_file_new)).area().rect(cx),
                "menu.file.open" => self.ui.button(cx, ids!(menu_file_open)).area().rect(cx),
                "menu.file.recent" | "menu.file.open_recent" => {
                    self.ui.button(cx, ids!(menu_file_recent)).area().rect(cx)
                }
                "menu.file.open_recent.submenu" => self
                    .ui
                    .widget(cx, ids!(file_recent_submenu))
                    .area()
                    .rect(cx),
                "recent.empty" => self.ui.label(cx, ids!(recent_empty)).area().rect(cx),
                "recent.item.0" => self.ui.button(cx, ids!(recent_file_0)).area().rect(cx),
                "recent.item.1" => self.ui.button(cx, ids!(recent_file_1)).area().rect(cx),
                "recent.item.2" => self.ui.button(cx, ids!(recent_file_2)).area().rect(cx),
                "recent.item.3" => self.ui.button(cx, ids!(recent_file_3)).area().rect(cx),
                "recent.item.4" => self.ui.button(cx, ids!(recent_file_4)).area().rect(cx),
                "recent.item.5" => self.ui.button(cx, ids!(recent_file_5)).area().rect(cx),
                "recent.item.6" => self.ui.button(cx, ids!(recent_file_6)).area().rect(cx),
                "recent.item.7" => self.ui.button(cx, ids!(recent_file_7)).area().rect(cx),
                "recent.item.8" => self.ui.button(cx, ids!(recent_file_8)).area().rect(cx),
                "recent.item.9" => self.ui.button(cx, ids!(recent_file_9)).area().rect(cx),
                "recent.clear" => self.ui.button(cx, ids!(recent_clear)).area().rect(cx),
                "menu.file.save" => self.ui.button(cx, ids!(menu_file_save)).area().rect(cx),
                "menu.file.save_as" => self.ui.button(cx, ids!(menu_file_save_as)).area().rect(cx),
                "menu.file.save_pack" => self
                    .ui
                    .button(cx, ids!(menu_file_save_pack))
                    .area()
                    .rect(cx),
                "menu.file.close" => self.ui.button(cx, ids!(menu_file_close)).area().rect(cx),
                "menu.file.exit" => self.ui.button(cx, ids!(menu_file_exit)).area().rect(cx),
                "context_menu" => self.ui.widget(cx, ids!(context_menu)).area().rect(cx),
                "context.copy" => self.ui.button(cx, ids!(context_copy)).area().rect(cx),
                "context.cut" => self.ui.button(cx, ids!(context_cut)).area().rect(cx),
                "context.paste" => self.ui.button(cx, ids!(context_paste)).area().rect(cx),
                "context.duplicate" => self.ui.button(cx, ids!(context_duplicate)).area().rect(cx),
                "context.delete" => self.ui.button(cx, ids!(context_delete)).area().rect(cx),
                "settings.close" => self.ui.button(cx, ids!(settings_close)).area().rect(cx),
                "about.close" => self.ui.button(cx, ids!(about_close)).area().rect(cx),
                "shortcuts.close" => self.ui.button(cx, ids!(shortcuts_close)).area().rect(cx),
                _ => self.ui.button(cx, ids!(menu_file)).area().rect(cx),
            };
            screen.rect(rect)
        };
        for (id, label) in [
            ("file_menu", "File menu"),
            ("edit_menu", "Edit menu"),
            ("view_menu", "View menu"),
            ("window_menu", "Window menu"),
            ("language_menu", "Language menu"),
            ("help_menu", "Help menu"),
            ("settings_panel", "Settings panel"),
            ("about_panel", "About panel"),
            ("shortcuts_panel", "Keyboard shortcuts panel"),
            ("resources_panel", "Resources panel"),
            ("layers_panel", "Layers panel"),
            ("menu.file.new", "New"),
            ("menu.file.open", "Open project"),
            ("menu.file.recent", "Open recent (legacy ID)"),
            ("menu.file.open_recent", "Open Recent"),
            ("menu.file.open_recent.submenu", "Open Recent submenu"),
            ("recent.empty", "No recent files"),
            ("recent.clear", "Clear Recent"),
            ("menu.file.save", "Save"),
            ("menu.file.save_as", "Save as"),
            ("menu.file.save_pack", "Save and pack"),
            ("menu.file.close", "Close document"),
            ("menu.file.exit", "Exit"),
            ("menu.edit.undo", "Undo"),
            ("menu.edit.redo", "Redo"),
            ("menu.edit.cut", "Cut"),
            ("menu.edit.copy", "Copy"),
            ("menu.edit.paste", "Paste"),
            ("menu.edit.duplicate", "Duplicate"),
            ("menu.edit.delete", "Delete"),
            ("menu.edit.select_all", "Select all"),
            ("context_menu", "Context menu"),
            ("context.copy", "Copy selected object"),
            ("context.cut", "Cut selected object"),
            ("context.paste", "Paste object"),
            ("context.duplicate", "Duplicate selected object"),
            ("context.delete", "Delete selected object"),
            ("menu.settings", "Settings"),
            ("settings.restore", "Restore session"),
            ("settings.close", "Close settings"),
            ("about.close", "Close about"),
            ("shortcuts.close", "Close keyboard shortcuts"),
            ("menu.view.zoom_in", "Zoom in"),
            ("menu.view.zoom_out", "Zoom out"),
            ("menu.view.fit", "Fit canvas"),
            ("menu.view.resources", "Toggle resources"),
            ("menu.view.layers", "Toggle layers"),
            ("menu.window.next", "Next document"),
            ("menu.window.previous", "Previous document"),
            ("menu.window.close", "Close document"),
            ("menu.window.close_others", "Close other documents"),
            ("menu.window.close_all", "Close all documents"),
            ("menu.language.english", "English"),
            ("menu.language.russian", "Russian"),
            ("menu.help.about", "About GLYF Editor"),
            ("menu.help.shortcuts", "Keyboard shortcuts"),
            ("dialog.open", "Open project dialog"),
            ("dialog.save_as", "Save project dialog"),
            ("clipboard.cut", "Cut selected object"),
            ("clipboard.copy", "Copy selected object"),
            ("clipboard.paste", "Paste object"),
            ("clipboard.duplicate", "Duplicate selected object"),
        ] {
            let mut node = accessibility_button_node(id, label, shell_control_bounds(id));
            let mut visible = self.shell_accessibility_visible(id);
            if id == "recent.empty" {
                visible &= self.settings.recent_files.is_empty();
            }
            node.visible = visible;
            node.enabled = visible
                && !matches!(id, "recent.empty" | "menu.file.open_recent.submenu")
                && self.menu_command_enabled(id).unwrap_or(true);
            shell_nodes.push(node);
        }
        for (index, entry) in desktop::recent_menu_entries(&self.settings.recent_files)
            .into_iter()
            .take(RECENT_MENU_CAPACITY)
            .enumerate()
        {
            let id = format!("recent.item.{index}");
            let mut node = accessibility_button_node(&id, &entry.label, shell_control_bounds(&id));
            node.visible = self.file_recent_submenu_open();
            node.enabled = node.visible;
            node.value = Some(entry.path);
            shell_nodes.push(node);
        }
        for (id, label, widget) in [
            ("menu.file", "File", ids!(menu_file)),
            ("menu.edit", "Edit", ids!(menu_edit)),
            ("menu.view", "View", ids!(menu_view)),
            ("menu.window", "Window", ids!(menu_window)),
            ("menu.language", "Language", ids!(menu_language)),
            ("menu.help", "Help", ids!(menu_help)),
        ] {
            shell_nodes.push(accessibility_button_node(
                id,
                label,
                screen.rect(self.ui.button(cx, widget).area().rect(cx)),
            ));
        }
        for (id, label, widget) in [
            ("window.minimize", "Minimize window", ids!(chrome_minimize)),
            (
                "window.maximize",
                "Maximize or restore window",
                ids!(chrome_maximize),
            ),
            ("window.close", "Close GLYF Editor", ids!(chrome_close)),
        ] {
            shell_nodes.push(accessibility_button_node(
                id,
                label,
                screen.rect(self.ui.button(cx, widget).area().rect(cx)),
            ));
        }
        let settings_visible = self.menu_open.as_deref() == Some("settings");
        for (id, label, category, widget) in [
            ("settings.category.general", "General", SettingsCategory::General, ids!(settings_category_general)),
            ("settings.category.appearance", "Appearance", SettingsCategory::Appearance, ids!(settings_category_appearance)),
            ("settings.category.editor", "Editor", SettingsCategory::Editor, ids!(settings_category_editor)),
            ("settings.category.files", "Files", SettingsCategory::Files, ids!(settings_category_files)),
            ("settings.category.accessibility", "Accessibility", SettingsCategory::Accessibility, ids!(settings_category_accessibility)),
        ] {
            let mut node = accessibility_button_node(
                id, label, screen.rect(self.ui.button(cx, widget).area().rect(cx)),
            );
            node.visible = settings_visible;
            node.enabled = settings_visible;
            node.selected = Some(self.settings_category == category);
            shell_nodes.push(node);
        }
        for (id, label, category, widget, value) in [
            ("settings.general.language", "Interface language", SettingsCategory::General, ids!(settings_language_select), self.settings.language.code().to_owned()),
            ("settings.general.recent_count", "Recent files count", SettingsCategory::General, ids!(settings_recent_count), self.settings.recent_file_count.to_string()),
            ("settings.appearance.preset", "Style preset", SettingsCategory::Appearance, ids!(settings_preset), StylePreset::ALL.iter().find(|preset| StyleProfile::from_preset(**preset) == self.style_profile).map(|preset| preset.code()).unwrap_or("custom").to_owned()),
            ("settings.appearance.palette", "Palette", SettingsCategory::Appearance, ids!(settings_palette), self.style_profile.palette.code().to_owned()),
            ("settings.appearance.shape", "Shape", SettingsCategory::Appearance, ids!(settings_shape), self.style_profile.shape.code().to_owned()),
            ("settings.appearance.surface", "Surface", SettingsCategory::Appearance, ids!(settings_surface), self.style_profile.surface.code().to_owned()),
            ("settings.appearance.motion", "Motion", SettingsCategory::Appearance, ids!(settings_motion), self.style_profile.motion.code().to_owned()),
            ("settings.appearance.font_size", "Interface font size", SettingsCategory::Appearance, ids!(settings_font_size), self.settings.font_size.code().to_owned()),
            ("settings.appearance.ui_scale", "UI scale", SettingsCategory::Appearance, ids!(settings_ui_scale), self.settings.ui_scale.code().to_owned()),
            ("settings.appearance.density", "Density", SettingsCategory::Appearance, ids!(settings_density), self.settings.density.code().to_owned()),
            ("settings.editor.default_zoom", "Default canvas zoom", SettingsCategory::Editor, ids!(settings_default_zoom), format!("{}", (self.settings.default_zoom * 100.0).round() as u16)),
        ] {
            let mut node = AccessibilityNode::new(
                id,
                AccessibilityRole::ComboBox,
                label,
                screen.rect(self.ui.drop_down(cx, widget).area().rect(cx)),
            );
            node.visible = settings_visible && self.settings_category == category;
            node.enabled = node.visible;
            node.value = Some(value);
            node.actions.set_value = true;
            shell_nodes.push(node);
        }
        for (id, label, category, widget, selected) in [
            ("settings.general.restore", "Restore previous session", SettingsCategory::General, ids!(settings_restore), self.settings.restore_session),
            ("settings.editor.resources", "Show Resources", SettingsCategory::Editor, ids!(settings_resources), self.panel_resources),
            ("settings.editor.layers", "Show Layers", SettingsCategory::Editor, ids!(settings_layers), self.panel_layers),
            ("settings.editor.status_bar", "Show Status Bar", SettingsCategory::Editor, ids!(settings_status_bar), self.settings.panel_status_bar),
            ("settings.editor.confirm_destructive", "Confirm destructive actions", SettingsCategory::Editor, ids!(settings_confirm_destructive), self.settings.confirm_destructive),
            ("settings.files.reopen_last", "Reopen last documents", SettingsCategory::Files, ids!(settings_reopen_last), self.settings.restore_session),
            ("settings.accessibility.reduced_motion", "Reduced Motion", SettingsCategory::Accessibility, ids!(settings_reduced_motion), matches!(self.style_profile.motion, MotionStyle::None | MotionStyle::Reduced)),
            ("settings.accessibility.large_font", "Large interface font", SettingsCategory::Accessibility, ids!(settings_large_font), self.settings.font_size == FontSizeChoice::Large),
        ] {
            let mut node = accessibility_button_node(
                id, label, screen.rect(self.ui.button(cx, widget).area().rect(cx)),
            );
            node.visible = settings_visible && self.settings_category == category;
            node.enabled = node.visible;
            node.selected = Some(selected);
            shell_nodes.push(node);
        }
        for (id, label, widget) in [
            ("settings.reset_appearance", "Reset Appearance", ids!(settings_reset_appearance)),
            ("settings.reset_all", "Reset All Settings", ids!(settings_reset_all)),
            ("settings.reset.yes", "Confirm reset", ids!(settings_reset_yes)),
            ("settings.reset.cancel", "Cancel reset", ids!(settings_reset_cancel)),
        ] {
            let mut node = accessibility_button_node(
                id, label, screen.rect(self.ui.button(cx, widget).area().rect(cx)),
            );
            node.visible = settings_visible && (!id.starts_with("settings.reset.")
                || self.ui.widget(cx, ids!(settings_reset_confirm)).visible());
            node.enabled = node.visible;
            shell_nodes.push(node);
        }
        let close_confirmation_visible = self.menu_open.as_deref() == Some("close_confirm");
        let mut close_confirmation = AccessibilityNode::new(
            "close_confirm_panel",
            AccessibilityRole::Pane,
            "Close document confirmation",
            screen.rect(
                self.ui
                    .widget(cx, ids!(close_confirm_panel))
                    .area()
                    .rect(cx),
            ),
        );
        close_confirmation.visible = close_confirmation_visible;
        close_confirmation.enabled = close_confirmation_visible;
        shell_nodes.push(close_confirmation);
        for (id, label, widget) in [
            (
                "tab.close.confirm.cancel",
                "Cancel close",
                ids!(close_confirm_cancel),
            ),
            (
                "tab.close.confirm.dont_save",
                "Close without saving",
                ids!(close_confirm_dont_save),
            ),
            (
                "tab.close.confirm.save",
                "Save and close",
                ids!(close_confirm_save),
            ),
        ] {
            let mut node = accessibility_button_node(
                id,
                label,
                screen.rect(self.ui.button(cx, widget).area().rect(cx)),
            );
            node.visible = close_confirmation_visible;
            node.enabled = close_confirmation_visible;
            shell_nodes.push(node);
        }
        let mut undo_node =
            accessibility_button_node("history.undo", "Undo last edit", shell_bounds);
        undo_node.value = Some(format!(
            "available={}; depth={}",
            !self.undo_stack.is_empty(),
            self.undo_stack.len()
        ));
        shell_nodes.push(undo_node);
        let mut redo_node =
            accessibility_button_node("history.redo", "Redo last edit", shell_bounds);
        redo_node.value = Some(format!(
            "available={}; depth={}",
            !self.redo_stack.is_empty(),
            self.redo_stack.len()
        ));
        shell_nodes.push(redo_node);
        let tab_strip_rect = self.document_tab_strip_rect(cx);
        let tab_bounds = screen.rect(tab_strip_rect);
        let russian = self.settings.language == Language::Russian;
        let mut tabs_node = AccessibilityNode::new(
            "tabs", AccessibilityRole::Pane,
            if russian { "Вкладки документов" } else { "Document tabs" }, tab_bounds);
        tabs_node.value = Some(format!(
            "count={}; active={}",
            self.tabs.len(),
            self.active_tab
        ));
        shell_nodes.push(tabs_node);
        shell_nodes.push(accessibility_text_node(
            "tab.count",
            if russian { "Открытые документы" } else { "Open documents" },
            tab_bounds,
            self.tabs.len().to_string(),
        ));
        shell_nodes.push(accessibility_text_node(
            "tab.active",
            if russian { "Активный документ" } else { "Active document" },
            tab_bounds,
            self.active_tab.to_string(),
        ));
        shell_nodes.push(accessibility_button_node(
            "tab.new",
            if russian { "Новая вкладка документа" } else { "New document tab" },
            screen.rect(self.ui.button(cx, ids!(tab_new)).area().rect(cx)),
        ));
        shell_nodes.push(accessibility_button_node(
            "tab.overflow",
            if russian { "Действия с вкладками" } else { "Document tab actions" },
            screen.rect(self.ui.button(cx, ids!(tab_overflow)).area().rect(cx)),
        ));
        let tab_frames = self.document_tab_frames();
        for (index, tab) in self.tabs.iter().enumerate() {
            let visual_rect = self.document_tab_rect(cx, tab).unwrap_or(tab_strip_rect);
            let mut node = accessibility_button_node(
                format!("tab.item.{index}"),
                tab_frames[index].accessibility_name_for(self.settings.language),
                screen.rect(visual_rect),
            );
            node.value = Some(if index == self.active_tab {
                format!("active; dirty={}", tab.dirty)
            } else {
                format!("dirty={}", tab.dirty)
            });
            shell_nodes.push(node);
            let close_rect = self.document_tab_close_rect(cx, tab).unwrap_or(Rect {
                pos: dvec2(visual_rect.pos.x + 4.0, visual_rect.pos.y + 3.0),
                size: dvec2(
                    visual_rect.size.x.min(20.0).max(1.0),
                    (visual_rect.size.y - 6.0).max(1.0),
                ),
            });
            shell_nodes.push(accessibility_button_node(
                format!("tab.item.{index}.close"),
                if russian { format!("Закрыть {}", tab_frames[index].title) } else { format!("Close {}", tab.title) },
                screen.rect(close_rect),
            ));
        }
        root.children.extend(shell_nodes);
        let mut edit_clip = AccessibilityNode::new(
            "part.edit_clip",
            AccessibilityRole::Button,
            "Edit selected source clip",
            screen.rect(self.ui.button(cx, ids!(part_edit_clip)).area().rect(cx)),
        );
        edit_clip.actions.invoke = true;
        root.children.push(editor_mode);
        root.children.push(project_dirty);
        root.children.push(selected_part_node);
        root.children.push(source_mode_node);
        root.children.push(source_image);
        root.children.push(source_width);
        root.children.push(source_height);
        root.children.push(canvas_zoom);
        root.children.push(canvas_pan);
        root.children.push(canvas_fit);
        root.children.push(canvas_100);
        root.children.push(canvas_zoom_out);
        root.children.push(canvas_zoom_in);
        root.children.push(source_zoom);
        root.children.push(source_pan);
        root.children.push(source_toolbar);
        root.children.push(source_surface);
        root.children.push(clip_selection);
        root.children.push(clip_input);
        root.children.push(clip_drag_active);
        root.children.push(clip_popup_node);
        root.children.push(part_popup_node);
        root.children.push(edit_clip);
        root.children.push(part_list);
        root.children.push(part_count);
        root.children.push(annotation_list);
        root.children.push(annotation_count);
        root.children.push(annotation_selected);
        root.children.push(annotation_role);
        root.children.push(annotation_geometry);
        root.children.push(annotation_comment_summary);
        root.children.push(region_drag_active);
        root.children.push(region_selection);
        root.children.push(region_pending);
        root.children.push(region_hover);
        root.children.push(region_input);
        root.children.extend(coordinate_nodes);
        root.children.push(annotation_popup);
        root.children.push(canvas);
        root.children.push(toolbar);
        root.children.push(status);
        root.children.push(image_list);
        root.children.push(image_count);
        root.children.push(preview);
        AccessibilityTree::new(root)
            .map(|tree| tree.with_generation(generation))
            .map_err(|error| error.to_string())
    }
}

#[cfg(windows)]
fn accessibility_button_node(
    id: impl Into<String>,
    name: impl Into<String>,
    bounds: AccessibilityRect,
) -> AccessibilityNode {
    let mut node = AccessibilityNode::new(id, AccessibilityRole::Button, name, bounds);
    node.actions.invoke = true;
    node
}
fn accessibility_text_node(
    id: impl Into<String>,
    name: impl Into<String>,
    bounds: AccessibilityRect,
    value: impl Into<String>,
) -> AccessibilityNode {
    let mut node = AccessibilityNode::new(id, AccessibilityRole::Text, name, bounds);
    node.value = Some(value.into());
    node
}

#[cfg(windows)]
fn annotation_coordinate_nodes(
    bounds: AccessibilityRect,
    annotation: Option<&GlyfAnnotation>,
) -> Vec<AccessibilityNode> {
    let mut values = [
        ("annotation.rel.x", "Rel X", "".to_owned()),
        ("annotation.rel.y", "Rel Y", "".to_owned()),
        ("annotation.rel1.x", "Rel1 X", "".to_owned()),
        ("annotation.rel1.y", "Rel1 Y", "".to_owned()),
        ("annotation.rel2.x", "Rel2 X", "".to_owned()),
        ("annotation.rel2.y", "Rel2 Y", "".to_owned()),
    ];
    if let Some(annotation) = annotation {
        match &annotation.geometry {
            GlyfAnnotationGeometry::Point { rel } => {
                values[0].2 = format!("{:.6}", rel.x);
                values[1].2 = format!("{:.6}", rel.y);
            }
            GlyfAnnotationGeometry::Region { rel1, rel2 } => {
                values[2].2 = format!("{:.6}", rel1.x);
                values[3].2 = format!("{:.6}", rel1.y);
                values[4].2 = format!("{:.6}", rel2.x);
                values[5].2 = format!("{:.6}", rel2.y);
            }
        }
    }
    values
        .into_iter()
        .map(|(id, name, value)| accessibility_text_node(id, name, bounds, value))
        .collect()
}
