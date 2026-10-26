//! Point/region annotation creation and draft lifecycle.

use super::*;

impl App {
    pub(crate) fn pending_annotation_document(&self) -> Option<&GlyfAnnotation> {
        match self.pending_annotation.as_ref() {
            Some(PendingAnnotation::Create { annotation }) => Some(annotation),
            Some(PendingAnnotation::Edit { index }) => self.annotations.get(*index),
            None => None,
        }
    }

    pub(crate) fn set_region_from_accessibility(&mut self, cx: &mut Cx, value: &str) {
        let values = value
            .split(|ch: char| ch == ',' || ch == ';' || ch.is_ascii_whitespace())
            .filter(|part| !part.is_empty())
            .filter_map(|part| part.parse::<f32>().ok())
            .collect::<Vec<_>>();
        if values.len() != 4 || values.iter().any(|value| !value.is_finite()) {
            self.set_status(cx, "Region expects rel1.x,rel1.y,rel2.x,rel2.y values");
            return;
        }
        let Some((rel1, rel2)) = normalize_region(
            GlyfPoint {
                x: values[0],
                y: values[1],
            },
            GlyfPoint {
                x: values[2],
                y: values[3],
            },
        ) else {
            self.set_status(cx, "Region rectangle is empty after normalization");
            return;
        };
        if self.pending_annotation.is_some() {
            self.set_status(cx, "Finish or cancel the current annotation first");
            return;
        }
        let before = self.capture_document_snapshot();
        if let Some(index) = self.selected_annotation {
            if let Some(annotation) = self.annotations.get_mut(index) {
                if matches!(&annotation.geometry, GlyfAnnotationGeometry::Region { .. }) {
                    annotation.geometry = GlyfAnnotationGeometry::Region { rel1, rel2 };
                    self.mark_dirty(cx);
                    self.sync_spatial(cx);

                    self.record_history("Update region geometry", before);
                    self.set_status(cx, "Semantic region geometry updated");
                    return;
                }
            }
        }
        self.set_canvas_tool(cx, CanvasTool::Region);
        self.selected_part = None;
        self.begin_region_annotation(cx, rel1, rel2);
    }

    pub(crate) fn annotation_anchor(annotation: &GlyfAnnotation) -> GlyfPoint {
        match &annotation.geometry {
            GlyfAnnotationGeometry::Point { rel } => *rel,
            GlyfAnnotationGeometry::Region { rel1, rel2 } => GlyfPoint {
                x: (rel1.x + rel2.x) * 0.5,
                y: (rel1.y + rel2.y) * 0.5,
            },
        }
    }

    pub(crate) fn annotation_popup_position(&self, cx: &Cx, annotation: &GlyfAnnotation) -> DVec2 {
        let content = zoomed_content_rect(
            self.canvas_rect(cx),
            self.selected_image_size(),
            self.canvas_zoom_value(),
            self.canvas_pan,
        );
        let marker =
            content.pos + relative_to_canvas(content.size, Self::annotation_anchor(annotation));
        let popup_width = 310.0;
        let popup_height = 128.0;
        let mut x = marker.x + 14.0;
        let mut y = marker.y + 14.0;
        if x + popup_width > content.pos.x + content.size.x {
            x = marker.x - popup_width - 14.0;
        }
        if y + popup_height > content.pos.y + content.size.y {
            y = marker.y - popup_height - 14.0;
        }
        dvec2(
            x.clamp(
                content.pos.x + 6.0,
                (content.pos.x + content.size.x - popup_width - 6.0).max(content.pos.x + 6.0),
            ),
            y.clamp(
                content.pos.y + 6.0,
                (content.pos.y + content.size.y - popup_height - 6.0).max(content.pos.y + 6.0),
            ),
        )
    }

    pub(crate) fn show_annotation_popup(&mut self, cx: &mut Cx, annotation: &GlyfAnnotation, editing: bool) {
        let text = annotation.comment.clone();
        let popup_pos = self.annotation_popup_position(cx, annotation);
        self.annotation_draft = text.clone();
        let region = matches!(&annotation.geometry, GlyfAnnotationGeometry::Region { .. });
        let russian = self.settings.language == Language::Russian;
        self.ui.label(cx, ids!(annotation_popup.title)).set_text(
            cx,
            &if russian && editing {
                format!("Изменить комментарий к {}", if region { "области" } else { "точке" })
            } else if russian {
                format!("Добавить {}", if region { "область" } else { "точку" })
            } else if editing {
                format!("Edit {} comment", if region { "region" } else { "point" })
            } else {
                format!("Add {} annotation", if region { "region" } else { "point" })
            },
        );
        self.ui
            .button(cx, ids!(annotation_popup.actions.commit))
            .set_text(cx, desktop::text("editor.done", self.settings.language));
        self.ui
            .text_input(cx, ids!(annotation_popup.comment_input))
            .set_text(cx, &text);
        if let Some(mut popup) = self.ui.view(cx, ids!(annotation_popup)).borrow_mut() {
            popup.walk.abs_pos = Some(popup_pos);
        }
        self.ui
            .view(cx, ids!(annotation_popup))
            .set_visible(cx, true);
        // A hidden View has no valid input Area before its first draw. Defer
        // keyboard focus until the normal overlay pass has produced it.
        self.annotation_popup_focus_on_next_draw = true;
    }
    pub(crate) fn begin_annotation(&mut self, cx: &mut Cx, rel: GlyfPoint) {
        if self.pending_annotation.is_some() {
            self.set_status(cx, "Finish or cancel the current annotation first");
            return;
        }
        let name = unique_annotation_name(
            &self.annotations,
            &format!("annotation_{}", self.annotations.len() + 1),
        );
        let annotation = GlyfAnnotation::point(name, rel, String::new());
        self.selected_annotation = None;
        self.pending_annotation = Some(PendingAnnotation::Create {
            annotation: annotation.clone(),
        });
        self.sync_spatial(cx);

        self.show_annotation_popup(cx, &annotation, false);
        self.set_status(cx, "Point ready — enter a comment, then Add");
    }

    pub(crate) fn begin_region_annotation(&mut self, cx: &mut Cx, rel1: GlyfPoint, rel2: GlyfPoint) {
        if self.pending_annotation.is_some() {
            self.set_status(cx, "Finish or cancel the current annotation first");
            return;
        }
        let Some((rel1, rel2)) = normalize_region(rel1, rel2) else {
            self.set_status(cx, "Region drag is too small");
            return;
        };
        let name = unique_annotation_name(
            &self.annotations,
            &format!("annotation_{}", self.annotations.len() + 1),
        );
        let annotation = GlyfAnnotation::region(name, rel1, rel2, String::new());
        self.selected_annotation = None;
        self.pending_annotation = Some(PendingAnnotation::Create {
            annotation: annotation.clone(),
        });
        self.sync_spatial(cx);

        self.show_annotation_popup(cx, &annotation, false);
        self.set_status(cx, "Region ready — enter a comment, then Add");
    }

    pub(crate) fn begin_edit_annotation(&mut self, cx: &mut Cx, index: usize) {
        if self.pending_annotation.is_some() || index >= self.annotations.len() {
            return;
        }
        let annotation = self.annotations[index].clone();
        self.selected_annotation = Some(index);
        self.pending_annotation = Some(PendingAnnotation::Edit { index });
        self.sync_spatial(cx);

        self.show_annotation_popup(cx, &annotation, true);
        self.set_status(cx, "Editing annotation comment");
    }

    pub(crate) fn hide_annotation_popup(&mut self, cx: &mut Cx) {
        self.ui
            .view(cx, ids!(annotation_popup))
            .set_visible(cx, false);
        self.annotation_popup_focus_on_next_draw = false;
        cx.revert_key_focus();
        cx.unblock_scrolling();
    }
    pub(crate) fn commit_annotation(&mut self, cx: &mut Cx) {
        let Some(pending) = self.pending_annotation.clone() else {
            return;
        };
        let popup_comment = self
            .ui
            .text_input(cx, ids!(annotation_popup.comment_input))
            .text();
        let comment = popup_comment.trim().to_owned();
        if comment.is_empty() {
            self.set_status(cx, "Annotation comment cannot be empty");
            return;
        }

        let before = self.capture_document_snapshot();
        let history_label = match pending {
            PendingAnnotation::Create { mut annotation } => {
                annotation.comment = comment.clone();
                let index = self.annotations.len();
                self.annotations.push(annotation);
                self.selected_annotation = Some(index);
                "Create annotation"
            }
            PendingAnnotation::Edit { index } => {
                let Some(annotation) = self.annotations.get_mut(index) else {
                    self.pending_annotation = None;
                    self.hide_annotation_popup(cx);
                    self.sync_spatial(cx);
                    self.set_status(cx, "Annotation no longer exists");
                    return;
                };
                annotation.comment = comment.clone();
                self.selected_annotation = Some(index);
                "Edit annotation comment"
            }
        };

        self.annotation_draft = comment;
        self.pending_annotation = None;
        self.hide_annotation_popup(cx);
        self.mark_dirty(cx);
        self.sync_spatial(cx);
        self.record_history(history_label, before);
        self.set_status(cx, "Annotation saved");
    }

    pub(crate) fn cancel_annotation(&mut self, cx: &mut Cx) {
        let Some(pending) = self.pending_annotation.take() else {
            return;
        };
        if matches!(pending, PendingAnnotation::Create { .. }) {
            self.selected_annotation = None;
        }
        self.annotation_draft.clear();
        self.hide_annotation_popup(cx);
        self.sync_spatial(cx);
        self.set_status(cx, "Annotation cancelled");
    }

    pub(crate) fn delete_annotation(&mut self, cx: &mut Cx) {
        if self.pending_annotation.is_some() {
            self.cancel_annotation(cx);
            return;
        }
        let Some(index) = self.selected_annotation else {
            self.set_status(cx, "Select an annotation first");
            return;
        };
        let before = self.capture_document_snapshot();
        if index < self.annotations.len() {
            self.annotations.remove(index);
        }
        self.selected_annotation = if self.annotations.is_empty() {
            None
        } else {
            Some(index.min(self.annotations.len() - 1))
        };
        self.mark_dirty(cx);
        self.sync_spatial(cx);
        self.record_history("Delete annotation", before);
        self.set_status(cx, "Annotation deleted");
    }
    pub(crate) fn handle_annotation_canvas_action(&mut self, cx: &mut Cx, action: AnnotationCanvasAction) {
        match action {
            AnnotationCanvasAction::Create { rel } => self.begin_annotation(cx, rel),
            AnnotationCanvasAction::CreateRegion { rel1, rel2 } => {
                self.begin_region_annotation(cx, rel1, rel2)
            }
            AnnotationCanvasAction::Select { index } => {
                self.selected_annotation = index;
                if index.is_some() {
                    self.selected_part = None;
                }
                self.sync_spatial(cx);

                self.set_status(
                    cx,
                    if index.is_some() {
                        "Annotation selected"
                    } else {
                        "Canvas selection cleared"
                    },
                );
            }
            AnnotationCanvasAction::Move {
                index,
                rel,
                final_change,
            } => {
                self.begin_gesture_history();
                if let Some(annotation) = self.annotations.get_mut(index) {
                    if let GlyfAnnotationGeometry::Point { rel: current } = &mut annotation.geometry
                    {
                        *current = rel;
                        self.selected_annotation = Some(index);
                        self.selected_part = None;
                    }
                }
                if final_change {
                    self.mark_dirty(cx);
                    self.set_status(cx, "Point moved");
                    self.finish_gesture_history("Move point");
                }
                self.sync_spatial(cx);
            }
            AnnotationCanvasAction::UpdateRegion {
                index,
                rel1,
                rel2,
                final_change,
            } => {
                self.begin_gesture_history();
                if let Some(annotation) = self.annotations.get_mut(index) {
                    if let GlyfAnnotationGeometry::Region {
                        rel1: current1,
                        rel2: current2,
                    } = &mut annotation.geometry
                    {
                        *current1 = rel1;
                        *current2 = rel2;
                        self.selected_annotation = Some(index);
                        self.selected_part = None;
                    }
                }
                if final_change {
                    self.mark_dirty(cx);
                    self.set_status(cx, "Region geometry updated");
                    self.finish_gesture_history("Resize region");
                }
                self.sync_spatial(cx);
            }
        }
    }
}
