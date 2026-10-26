//! Model-to-canvas projection, viewport transforms and selection.

use super::*;

impl App {
    pub(crate) fn refresh(&mut self, cx: &mut Cx) {
        let language = self.settings.language;
        let list = if self.images.is_empty() {
            desktop::text("editor.no_images_yet", language).to_owned()
        } else {
            self.images
                .iter()
                .enumerate()
                .map(|(index, image)| {
                    let marker = if Some(index) == self.selected {
                        "●"
                    } else {
                        "○"
                    };
                    format!("{marker} {}", image.name)
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        self.ui.label(cx, ids!(images_list)).set_text(cx, &list);

        let labels = if self.images.is_empty() {
            vec![desktop::text("editor.no_images", language).to_owned()]
        } else {
            self.images.iter().map(|image| image.name.clone()).collect()
        };
        self.ui
            .drop_down(cx, ids!(image_dropdown))
            .set_labels(cx, labels);
        if let Some(selected) = self.selected {
            self.ui
                .drop_down(cx, ids!(image_dropdown))
                .set_selected_item(cx, selected);
            if let Some(image) = self.images.get(selected) {
                self.ui
                    .text_input(cx, ids!(name_input))
                    .set_text(cx, &image.name);
                self.ui
                    .text_input(cx, ids!(comment_input))
                    .set_text(cx, &image.comment);
                self.ui
                    .label(cx, ids!(source_label))
                    .set_text(cx, &format!("{} {}", desktop::text("editor.source_label", language), image.source));
                self.ui.image(cx, ids!(preview_image)).set_visible(cx, true);
                self.ui
                    .label(cx, ids!(preview_placeholder))
                    .set_visible(cx, false);
            }
        } else {
            self.ui.text_input(cx, ids!(name_input)).set_text(cx, "");
            self.ui.text_input(cx, ids!(comment_input)).set_text(cx, "");
            self.ui
                .label(cx, ids!(source_label))
                .set_text(cx, &format!("{} —", desktop::text("editor.source_label", language)));
            self.ui
                .image(cx, ids!(preview_image))
                .set_visible(cx, false);
            self.ui
                .label(cx, ids!(preview_placeholder))
                .set_visible(cx, true);
        }
        let composition_visible = !self.source_mode;
        self.ui
            .image(cx, ids!(preview_image))
            .set_visible(cx, composition_visible);
        self.ui
            .widget(cx, ids!(canvas_overlay))
            .set_visible(cx, composition_visible);
        self.ui
            .widget(cx, ids!(annotation_toolbar))
            .set_visible(cx, composition_visible);
        self.ui
            .widget(cx, ids!(source_canvas))
            .set_visible(cx, !composition_visible);
        self.ui
            .widget(cx, ids!(source_toolbar))
            .set_visible(cx, !composition_visible);

        self.ui.label(cx, ids!(mode_badge)).set_text(
            cx,
            &if self.source_mode {
                format!("{} · {}", desktop::text("editor.source_mode", language), self.selected_image_name())
            } else {
                desktop::text("editor.composition", language).to_owned()
            },
        );
        self.ui.label(cx, ids!(chrome_title)).set_text(
            cx,
            if self.dirty {
                "GLYF Editor *"
            } else {
                WINDOW_TITLE
            },
        );
        self.ui
            .label(cx, ids!(dirty_badge))
            .set_text(cx, desktop::text(if self.dirty { "editor.unsaved" } else { "editor.saved" }, language));
        self.sync_source_canvas(cx);
        self.ui.widget(cx, ids!(preview)).redraw(cx);
        self.sync_spatial(cx);
    }

    pub(crate) fn current_parts(&self) -> &[GlyfPart] {
        self.groups
            .get(0)
            .map(|group| group.parts.as_slice())
            .unwrap_or(&[])
    }

    pub(crate) fn current_parts_mut(&mut self) -> &mut Vec<GlyfPart> {
        if self.groups.is_empty() {
            self.groups.push(GlyfGroup {
                name: "main".into(),
                parts: Vec::new(),
            });
        }
        &mut self.groups[0].parts
    }

    pub(crate) fn selected_source_size(&self) -> Option<DVec2> {
        self.selected
            .and_then(|index| self.images.get(index))
            .and_then(|image| image.pixel_size)
    }

    pub(crate) fn sync_source_canvas(&mut self, cx: &mut Cx) {
        let language = self.settings.language;
        let russian = language == Language::Russian;
        let selected_clip = self
            .pending_clip
            .as_ref()
            .map(|pending| pending.clip)
            .or(self.source_clip)
            .or_else(|| {
                self.selected_part
                    .and_then(|index| self.current_parts().get(index).and_then(|part| part.clip))
            });
        if let Some(mut canvas) = self
            .ui
            .widget(cx, ids!(source_canvas))
            .borrow_mut::<SourceCanvas>()
        {
            canvas.set_tool(cx, self.source_tool);
            canvas.set_view(cx, self.source_zoom_value(), self.source_pan);
            canvas.set_clip(cx, selected_clip);
        }
        let tokens = self.style_profile.tokens(StyleTuning::default());
        for (id, tool) in [(ids!(source_select), SourceTool::Select), (ids!(source_clip), SourceTool::Clip)] {
            let appearance = tokens.button(ButtonVariant::Secondary);
            let fill = if self.source_tool == tool { tokens.accent_subtle } else { appearance.fill };
            let border = if self.source_tool == tool { tokens.accent } else { appearance.border };
            let mut button = self.ui.button(cx, id);
            button.set_text(cx, "");
            script_apply_eval!(cx, button, { draw_bg +: { color: #(fill) border_color: #(border) } });
        }
        self.ui
            .label(cx, ids!(source_zoom_label))
            .set_text(cx, &format!("{:.0}%", self.source_zoom_value() * 100.0));
        if let Some(size) = self.selected_source_size() {
            let clip_text = selected_clip
                .map(|clip| {
                    format!(
                        "{} {:.0}, {:.0}, {:.0}, {:.0}",
                        if russian { "Фрагмент" } else { "Clip" },
                        clip.x, clip.y, clip.w, clip.h
                    )
                })
                .unwrap_or_else(|| if russian { "Фрагмент —" } else { "Clip —" }.into());
            self.ui.label(cx, ids!(source_status)).set_text(
                cx,
                &format!(
                    "{} · {:.0} × {:.0} px · {} · {} {:.0}% · {} {:.0},{:.0}",
                    self.selected_image_name(),
                    size.x,
                    size.y,
                    clip_text,
                    if russian { "Масштаб" } else { "Zoom" },
                    self.source_zoom_value() * 100.0,
                    if russian { "Смещение" } else { "Pan" },
                    self.source_pan.x,
                    self.source_pan.y,
                ),
            );
        } else {
            self.ui
                .label(cx, ids!(source_status))
                .set_text(cx, &format!("{} —", desktop::text("editor.source_label", language)));
        }
    }
    pub(crate) fn selected_image_name(&self) -> String {
        self.selected
            .and_then(|index| self.images.get(index))
            .map(|image| image.name.clone())
            .unwrap_or_else(|| "—".into())
    }

    pub(crate) fn source_zoom_value(&self) -> f64 {
        if self.source_zoom <= 0.0 {
            1.0
        } else {
            self.source_zoom.clamp(MIN_VIEW_ZOOM, MAX_VIEW_ZOOM)
        }
    }
    pub(crate) fn apply_tool_button_selection(&mut self, cx: &mut Cx) {
        let tokens = self.style_profile.tokens(StyleTuning::default());
        for (id, tool) in [
            (ids!(tool_select), CanvasTool::Select),
            (ids!(tool_point), CanvasTool::Point),
            (ids!(tool_region), CanvasTool::Region),
        ] {
            let appearance = tokens.button(ButtonVariant::Secondary);
            let selected = self.tool == tool;
            let fill = if selected { tokens.accent_subtle } else { appearance.fill };
            let hover = appearance.hover;
            let pressed = appearance.pressed;
            let border = if selected { tokens.accent } else { appearance.border };
            let label = appearance.text;
            let mut button = self.ui.button(cx, id);
            script_apply_eval!(cx, button, {
                draw_bg +: {
                    color: #(fill) color_hover: #(hover) color_down: #(pressed)
                    border_color: #(border)
                }
                draw_text +: { color: #(label) color_hover: #(label) }
            });
        }
    }
    pub(crate) fn sync_spatial(&mut self, cx: &mut Cx) {
        let language = self.settings.language;
        self.apply_tool_button_selection(cx);
        self.ui
            .label(cx, ids!(canvas_zoom_label))
            .set_text(cx, &format!("{:.0}%", self.canvas_zoom_value() * 100.0));
        let selected_part = self
            .selected_part
            .filter(|index| *index < self.current_parts().len());
        if self.selected_part != selected_part {
            self.selected_part = selected_part;
        }
        let parts = self.current_parts().to_vec();
        let layer_text = if parts.is_empty() {
            desktop::text("editor.preview_empty", language).to_owned()
        } else {
            parts
                .iter()
                .enumerate()
                .map(|(index, part)| {
                    let marker = if Some(index) == selected_part {
                        "●"
                    } else {
                        "○"
                    };
                    format!("{marker} {}  [{}]", part.name, part.source)
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        self.ui
            .label(cx, ids!(layers_list))
            .set_text(cx, &layer_text);
        let part_labels = if parts.is_empty() {
            vec![desktop::text("editor.no_parts", language).to_owned()]
        } else {
            parts.iter().map(|part| part.name.clone()).collect()
        };
        self.ui
            .drop_down(cx, ids!(part_dropdown))
            .set_labels(cx, part_labels);
        if let Some(index) = selected_part {
            self.ui
                .drop_down(cx, ids!(part_dropdown))
                .set_selected_item(cx, index);
            let part = &parts[index];
            self.ui
                .text_input(cx, ids!(part_name_input))
                .set_text(cx, &part.name);
            self.ui
                .text_input(cx, ids!(part_source_input))
                .set_text(cx, &part.source);
            self.ui
                .text_input(cx, ids!(part_parent_input))
                .set_text(cx, part.parent.as_deref().unwrap_or(""));
            let clip = part.clip.unwrap_or(GlyfClip {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
            });
            for (id, value) in [
                (ids!(clip_x_input), clip.x),
                (ids!(clip_y_input), clip.y),
                (ids!(clip_w_input), clip.w),
                (ids!(clip_h_input), clip.h),
                (ids!(rel1_x_input), part.rel1.x),
                (ids!(rel1_y_input), part.rel1.y),
                (ids!(rel2_x_input), part.rel2.x),
                (ids!(rel2_y_input), part.rel2.y),
            ] {
                self.ui
                    .text_input(cx, id)
                    .set_text(cx, &format!("{value:.4}"));
            }
            self.ui
                .text_input(cx, ids!(part_comment_input))
                .set_text(cx, &part.comment);
        } else {
            for id in [
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
            ] {
                self.ui.text_input(cx, id).set_text(cx, "");
            }
        }
        self.selected_annotation = self
            .selected_annotation
            .filter(|index| *index < self.annotations.len());
        let annotation_text = if self.annotations.is_empty() {
            desktop::text("editor.no_annotations_yet", language).to_owned()
        } else {
            self.annotations
                .iter()
                .enumerate()
                .map(|(index, annotation)| {
                    let marker = if Some(index) == self.selected_annotation {
                        "●"
                    } else {
                        "○"
                    };
                    format!(
                        "{marker} {} [{}] — {}",
                        annotation.name,
                        localized_annotation_geometry_summary(annotation, language),
                        annotation.comment
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        self.ui
            .label(cx, ids!(annotations_list))
            .set_text(cx, &annotation_text);
        let annotation_labels = if self.annotations.is_empty() {
            vec![desktop::text("editor.no_annotations", language).to_owned()]
        } else {
            self.annotations
                .iter()
                .map(|annotation| annotation.name.clone())
                .collect()
        };
        self.ui
            .drop_down(cx, ids!(annotation_dropdown))
            .set_labels(cx, annotation_labels);
        if let Some(index) = self.selected_annotation {
            self.ui
                .drop_down(cx, ids!(annotation_dropdown))
                .set_selected_item(cx, index);
        }

        let selected_image_size = self
            .selected
            .and_then(|index| self.images.get(index))
            .and_then(|image| image.pixel_size);
        self.ui.label(cx, ids!(annotation_geometry)).set_text(
            cx,
            &self
                .selected_annotation
                .and_then(|index| self.annotations.get(index))
                .map(|annotation| localized_annotation_geometry_summary(annotation, language))
                .unwrap_or_else(|| desktop::text("editor.no_selection", language).into()),
        );
        self.ui.label(cx, ids!(annotation_role)).set_text(
            cx,
            &self
                .selected_annotation
                .and_then(|index| self.annotations.get(index))
                .map(|annotation| format!("{} {}", desktop::text("editor.role_label", language), annotation.role))
                .unwrap_or_else(|| format!("{} —", desktop::text("editor.role_label", language))),
        );
        self.ui.label(cx, ids!(annotation_rel)).set_text(
            cx,
            &self
                .selected_annotation
                .and_then(|index| self.annotations.get(index))
                .map(|annotation| format!("{} {}", desktop::text("editor.coordinates_label", language), annotation_coordinates_summary(annotation)))
                .unwrap_or_else(|| format!("{} —", desktop::text("editor.coordinates_label", language))),
        );
        self.ui.label(cx, ids!(annotation_comment)).set_text(
            cx,
            &self
                .selected_annotation
                .and_then(|index| self.annotations.get(index))
                .map(|annotation| format!("{} {}", desktop::text("editor.comment_label", language), annotation.comment))
                .unwrap_or_else(|| format!("{} —", desktop::text("editor.comment_label", language))),
        );
        let atlas_sources = self.atlases_dirty.then(|| {
            self.images
                .iter()
                .map(|image| (image.name.clone(), image.bytes.clone()))
                .collect::<Vec<_>>()
        });
        let pending_annotation = self.pending_annotation_document().cloned();
        if let Some(mut canvas) = self
            .ui
            .widget(cx, ids!(canvas_overlay))
            .borrow_mut::<SpatialCanvas>()
        {
            canvas.set_parts(cx, &parts, selected_part);
            canvas.set_annotations(cx, &self.annotations, self.selected_annotation);
            canvas.set_pending_annotation(cx, pending_annotation.as_ref());
            canvas.set_tool(cx, self.tool);
            canvas.set_view(cx, self.canvas_zoom_value(), self.canvas_pan);
            canvas.set_image_size(cx, selected_image_size);
            if let Some(atlas_sources) = &atlas_sources {
                canvas.set_atlases(cx, atlas_sources);
            }
        }
        if atlas_sources.is_some() {
            self.atlases_dirty = false;
        }
    }
    // A viewport change must not rebuild object lists or overwrite metadata
    // inputs: it only changes the canvas transform and its zoom indicator.
    pub(crate) fn sync_canvas_view(&mut self, cx: &mut Cx) {
        self.ui
            .label(cx, ids!(canvas_zoom_label))
            .set_text(cx, &format!("{:.0}%", self.canvas_zoom_value() * 100.0));
        if let Some(mut canvas) = self
            .ui
            .widget(cx, ids!(canvas_overlay))
            .borrow_mut::<SpatialCanvas>()
        {
            canvas.set_view(cx, self.canvas_zoom_value(), self.canvas_pan);
        }
    }
    pub(crate) fn fit_canvas(&mut self, cx: &mut Cx) {
        self.canvas_zoom = 1.0;
        self.canvas_pan = DVec2::default();
        self.sync_canvas_view(cx);

        self.set_status(cx, "Canvas fit");
    }

    pub(crate) fn canvas_100(&mut self, cx: &mut Cx) {
        self.canvas_zoom = one_to_one_zoom(self.canvas_rect(cx), self.selected_image_size());
        self.canvas_pan = DVec2::default();
        self.sync_canvas_view(cx);

        self.set_status(cx, "Canvas 100% · 1 source px ≈ 1 logical px");
    }

    pub(crate) fn adjust_canvas_zoom(&mut self, cx: &mut Cx, factor: f64) {
        let viewport = self.canvas_rect(cx);
        let anchor = viewport.pos + viewport.size * 0.5;
        let view = zoom_view_around(
            viewport,
            self.selected_image_size(),
            self.canvas_zoom_value(),
            self.canvas_pan,
            anchor,
            self.canvas_zoom_value() * factor,
        );
        self.canvas_zoom = view.zoom;
        self.canvas_pan = view.pan;
        self.sync_canvas_view(cx);

        self.set_status(cx, format!("Canvas zoom {:.0}%", self.canvas_zoom * 100.0));
    }

    pub(crate) fn fit_source(&mut self, cx: &mut Cx) {
        self.source_zoom = 1.0;
        self.source_pan = DVec2::default();
        self.sync_source_canvas(cx);
        self.set_status(cx, "Source fit");
    }

    pub(crate) fn source_100(&mut self, cx: &mut Cx) {
        self.source_zoom =
            one_to_one_zoom(self.source_canvas_rect(cx), self.selected_source_size());
        self.source_pan = DVec2::default();
        self.sync_source_canvas(cx);
        self.set_status(cx, "Source 100% · 1 source px ≈ 1 logical px");
    }

    pub(crate) fn adjust_source_zoom(&mut self, cx: &mut Cx, factor: f64) {
        let viewport = self.source_canvas_rect(cx);
        let anchor = viewport.pos + viewport.size * 0.5;
        let view = zoom_view_around(
            viewport,
            self.selected_source_size(),
            self.source_zoom_value(),
            self.source_pan,
            anchor,
            self.source_zoom_value() * factor,
        );
        self.source_zoom = view.zoom;
        self.source_pan = view.pan;
        self.sync_source_canvas(cx);
        self.set_status(cx, format!("Source zoom {:.0}%", self.source_zoom * 100.0));
    }

    pub(crate) fn enter_source_mode(&mut self, cx: &mut Cx) {
        let Some(selected) = self.selected else {
            self.set_status(cx, "Select a resource before opening Source mode");
            return;
        };
        self.source_mode = true;
        self.source_tool = SourceTool::Select;
        self.source_zoom = 1.0;
        self.source_pan = DVec2::default();
        self.source_clip = self
            .selected_part
            .and_then(|index| self.current_parts().get(index))
            .filter(|part| part.source == self.images[selected].name)
            .and_then(|part| part.clip);
        self.refresh(cx);
        self.set_status(cx, format!("Source mode · {}", self.images[selected].name));
    }

    pub(crate) fn leave_source_mode(&mut self, cx: &mut Cx) {
        self.pending_clip = None;
        self.hide_clip_popup(cx);
        self.source_mode = false;
        self.source_tool = SourceTool::Select;
        self.source_clip = None;
        self.source_zoom = 1.0;
        self.source_pan = DVec2::default();
        self.refresh(cx);
        self.set_status(cx, "Back to composition");
    }

    pub(crate) fn set_source_tool(&mut self, cx: &mut Cx, tool: SourceTool) {
        self.source_tool = tool;

        self.sync_source_canvas(cx);
        self.set_status(
            cx,
            if tool == SourceTool::Clip {
                "Clip tool active — drag source pixels"
            } else {
                "Source select tool active"
            },
        );
    }

    pub(crate) fn selected_image_size(&self) -> Option<DVec2> {
        self.selected
            .and_then(|index| self.images.get(index))
            .and_then(|image| image.pixel_size)
    }

    pub(crate) fn set_canvas_tool(&mut self, cx: &mut Cx, tool: CanvasTool) {
        if self.pending_annotation.is_some() {
            self.cancel_annotation(cx);
        }
        self.tool = tool;
        self.sync_spatial(cx);

        self.set_status(
            cx,
            match tool {
                CanvasTool::Select => "Select tool active",
                CanvasTool::Point => "Point tool active — click the Canvas",
                CanvasTool::Region => "Region tool active — drag on the Canvas",
            },
        );
    }

    pub(crate) fn canvas_zoom_value(&self) -> f64 {
        if self.canvas_zoom <= 0.0 {
            1.0
        } else {
            self.canvas_zoom.clamp(MIN_VIEW_ZOOM, MAX_VIEW_ZOOM)
        }
    }

    pub(crate) fn handle_canvas_viewport_action(&mut self, cx: &mut Cx, action: CanvasViewportAction) {
        self.canvas_zoom = action.zoom.clamp(MIN_VIEW_ZOOM, MAX_VIEW_ZOOM);
        self.canvas_pan = action.pan;
        self.sync_canvas_view(cx);
    }

    pub(crate) fn handle_canvas_hover_action(&mut self, cx: &mut Cx, action: CanvasHoverAction) {
        self.hovered_annotation = action.annotation;
        self.hovered_part = action.part;
        if let Some(index) = action.annotation {
            if let Some(annotation) = self.annotations.get(index) {
                self.set_status(cx, format!("{} · {}", annotation.name, annotation.comment));
            }
        } else if let Some(index) = action.part {
            if let Some(part) = self.current_parts().get(index) {
                self.set_status(cx, format!("{} · {}", part.name, part.comment));
            }
        }
    }

    pub(crate) fn handle_canvas_edit_action(&mut self, cx: &mut Cx, action: CanvasEditAction) {
        match action {
            CanvasEditAction::Annotation { index } => self.begin_edit_annotation(cx, index),
            CanvasEditAction::Part { index } => self.begin_edit_part(cx, index),
        }
    }

    pub(crate) fn handle_canvas_action(&mut self, cx: &mut Cx, action: SpatialCanvasAction) {
        match action {
            SpatialCanvasAction::Select { index } => {
                self.selected_part = index;
                if index.is_some() {
                    self.selected_annotation = None;
                }
                self.sync_spatial(cx);

                self.set_status(
                    cx,
                    if index.is_some() {
                        "Part selected"
                    } else {
                        "Canvas selection cleared"
                    },
                );
            }
            SpatialCanvasAction::ContextMenu { pos } => self.open_canvas_context_menu(cx, pos),
            SpatialCanvasAction::Update {
                index,
                rel1,
                rel2,
                final_change,
            } => {
                self.begin_gesture_history();
                if let Some(part) = self.current_parts_mut().get_mut(index) {
                    part.rel1 = rel1;
                    part.rel2 = rel2;
                    self.selected_part = Some(index);
                    self.selected_annotation = None;
                }
                if final_change {
                    self.mark_dirty(cx);
                    self.set_status(cx, "Part geometry updated");
                    self.finish_gesture_history("Move or resize part");
                }
                self.sync_spatial(cx);
            }
        }
    }

    pub(crate) fn canvas_rect(&self, cx: &Cx) -> Rect {
        let preview = self.ui.view(cx, ids!(preview)).area().rect(cx);
        let canvas = self.ui.widget(cx, ids!(canvas_overlay)).area().rect(cx);
        if canvas.size.x > 2.0 && canvas.size.y > 2.0 {
            canvas
        } else {
            Rect {
                pos: dvec2(preview.pos.x, preview.pos.y + 52.0),
                size: dvec2(preview.size.x, (preview.size.y - 52.0).max(1.0)),
            }
        }
    }

    pub(crate) fn source_canvas_rect(&self, cx: &Cx) -> Rect {
        let source = self.ui.widget(cx, ids!(source_canvas)).area().rect(cx);
        if source.size.x > 2.0 && source.size.y > 2.0 {
            source
        } else {
            self.canvas_rect(cx)
        }
    }
}
