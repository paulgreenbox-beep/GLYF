//! Composition part editing, placement and z-order.

use super::*;

impl App {
    pub(crate) fn part_popup_position(&self, cx: &Cx, index: usize) -> DVec2 {
        let viewport = self.canvas_rect(cx);
        let content = zoomed_content_rect(
            viewport,
            self.selected_image_size(),
            self.canvas_zoom_value(),
            self.canvas_pan,
        );
        let anchor = self
            .current_parts()
            .get(index)
            .map(|part| GlyfPoint {
                x: (part.rel1.x + part.rel2.x) * 0.5,
                y: (part.rel1.y + part.rel2.y) * 0.5,
            })
            .unwrap_or(GlyfPoint { x: 0.5, y: 0.5 });
        let marker = content.pos + relative_to_canvas(content.size, anchor);
        let popup_width = 350.0;
        let popup_height = 170.0;
        let mut x = marker.x + 14.0;
        let mut y = marker.y + 14.0;
        if x + popup_width > viewport.pos.x + viewport.size.x {
            x = marker.x - popup_width - 14.0;
        }
        if y + popup_height > viewport.pos.y + viewport.size.y {
            y = marker.y - popup_height - 14.0;
        }
        dvec2(
            x.clamp(
                viewport.pos.x + 8.0,
                (viewport.pos.x + viewport.size.x - popup_width - 8.0).max(viewport.pos.x + 8.0),
            ),
            y.clamp(
                viewport.pos.y + 8.0,
                (viewport.pos.y + viewport.size.y - popup_height - 8.0).max(viewport.pos.y + 8.0),
            ),
        )
    }

    pub(crate) fn begin_edit_part(&mut self, cx: &mut Cx, index: usize) {
        if self.has_pending_work() || index >= self.current_parts().len() {
            self.set_status(cx, "Finish or cancel the current edit first");
            return;
        }
        let part = self.current_parts()[index].clone();
        self.begin_gesture_history();
        self.selected_part = Some(index);
        self.selected_annotation = None;
        self.pending_part_popup = Some(PendingPartPopup {
            index,
            original_name: part.name.clone(),
            original_comment: part.comment.clone(),
            draft_name: part.name.clone(),
            draft_comment: part.comment.clone(),
        });
        self.sync_spatial(cx);

        let pos = self.part_popup_position(cx, index);
        self.show_clip_popup_at(cx, "Edit part metadata", &part.name, &part.comment, pos);
        self.set_status(cx, "Editing part name and comment");
    }

    pub(crate) fn commit_part_popup(&mut self, cx: &mut Cx) {
        let Some(pending) = self.pending_part_popup.clone() else {
            return;
        };
        let (popup_name, popup_comment) = self
            .ui
            .widget(cx, ids!(clip_popup))
            .borrow_mut::<ClipPopup>()
            .map(|popup| {
                (
                    popup.name(cx).trim().to_owned(),
                    popup.comment(cx).trim().to_owned(),
                )
            })
            .unwrap_or_default();
        let name = if popup_name.is_empty() {
            pending.draft_name.clone()
        } else {
            popup_name
        };
        let comment = if popup_comment.is_empty() && !pending.draft_comment.is_empty() {
            pending.draft_comment.clone()
        } else {
            popup_comment
        };
        if name.is_empty() {
            self.set_status(cx, "Part name is required");
            return;
        }
        if self
            .current_parts()
            .iter()
            .enumerate()
            .any(|(other, part)| other != pending.index && part.name == name)
        {
            self.set_status(cx, "Part names must be unique within the group");
            return;
        }
        if let Some(part) = self.current_parts_mut().get_mut(pending.index) {
            part.name = name;
            part.comment = comment;
        }
        self.pending_part_popup = None;
        self.hide_clip_popup(cx);
        self.mark_dirty(cx);
        self.sync_spatial(cx);

        self.finish_gesture_history("Update part metadata");
        self.set_status(cx, "Part metadata saved");
    }

    pub(crate) fn cancel_part_popup(&mut self, cx: &mut Cx) {
        let Some(pending) = self.pending_part_popup.take() else {
            return;
        };
        self.hide_clip_popup(cx);
        self.sync_spatial(cx);

        self.set_status(
            cx,
            format!(
                "Part edit cancelled · {} · {}",
                pending.original_name, pending.original_comment
            ),
        );
    }
    pub(crate) fn add_part(&mut self, cx: &mut Cx, clipped: bool) {
        let Some(image_index) = self.selected else {
            self.set_status(cx, "Select a resource before adding a part");
            return;
        };
        let Some(image_name) = self.images.get(image_index).map(|image| image.name.clone()) else {
            return;
        };
        let before = self.capture_document_snapshot();
        let index = self.current_parts().len();
        let name = unique_part_name(self.current_parts(), &format!("part_{}", index + 1));
        let clip = if clipped {
            Some(GlyfClip {
                x: self
                    .ui
                    .text_input(cx, ids!(clip_x_input))
                    .text()
                    .parse()
                    .unwrap_or(0.0)
                    .max(0.0),
                y: self
                    .ui
                    .text_input(cx, ids!(clip_y_input))
                    .text()
                    .parse()
                    .unwrap_or(0.0)
                    .max(0.0),
                w: self
                    .ui
                    .text_input(cx, ids!(clip_w_input))
                    .text()
                    .parse()
                    .unwrap_or(160.0)
                    .max(1.0),
                h: self
                    .ui
                    .text_input(cx, ids!(clip_h_input))
                    .text()
                    .parse()
                    .unwrap_or(96.0)
                    .max(1.0),
            })
        } else {
            None
        };
        self.current_parts_mut().push(GlyfPart {
            name,
            source: image_name,
            clip,
            rel1: GlyfPoint {
                x: (0.08 + index as f32 * 0.04).min(0.7),
                y: (0.08 + index as f32 * 0.04).min(0.7),
            },
            rel2: GlyfPoint {
                x: (0.48 + index as f32 * 0.04).min(0.95),
                y: (0.42 + index as f32 * 0.04).min(0.95),
            },
            comment: String::new(),
            parent: None,
        });
        self.selected_part = Some(index);
        self.mark_dirty(cx);
        self.sync_spatial(cx);
        self.record_history(
            if clipped {
                "Add clipped part"
            } else {
                "Add part"
            },
            before,
        );

        self.set_status(
            cx,
            if clipped {
                "Clipped part created"
            } else {
                "Part created from source"
            },
        );
    }

    pub(crate) fn apply_part(&mut self, cx: &mut Cx) {
        let Some(index) = self.selected_part else {
            self.set_status(cx, "Select a part first");
            return;
        };
        let name = self
            .ui
            .text_input(cx, ids!(part_name_input))
            .text()
            .trim()
            .to_owned();
        let source = self
            .ui
            .text_input(cx, ids!(part_source_input))
            .text()
            .trim()
            .to_owned();
        if name.is_empty() || source.is_empty() {
            self.set_status(cx, "Part name and source are required");
            return;
        }
        if !self.images.iter().any(|image| image.name == source) {
            self.set_status(cx, "Part source must reference an existing image");
            return;
        }
        if self
            .current_parts()
            .iter()
            .enumerate()
            .any(|(other, part)| other != index && part.name == name)
        {
            self.set_status(cx, "Part names must be unique within the group");
            return;
        }

        let clip_x = self
            .ui
            .text_input(cx, ids!(clip_x_input))
            .text()
            .parse::<f32>()
            .unwrap_or(0.0)
            .max(0.0);
        let clip_y = self
            .ui
            .text_input(cx, ids!(clip_y_input))
            .text()
            .parse::<f32>()
            .unwrap_or(0.0)
            .max(0.0);
        let clip_w = self
            .ui
            .text_input(cx, ids!(clip_w_input))
            .text()
            .parse::<f32>()
            .unwrap_or(0.0)
            .max(0.0);
        let clip_h = self
            .ui
            .text_input(cx, ids!(clip_h_input))
            .text()
            .parse::<f32>()
            .unwrap_or(0.0)
            .max(0.0);
        let rel1_x = self
            .ui
            .text_input(cx, ids!(rel1_x_input))
            .text()
            .parse::<f32>()
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let rel1_y = self
            .ui
            .text_input(cx, ids!(rel1_y_input))
            .text()
            .parse::<f32>()
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let rel2_x = self
            .ui
            .text_input(cx, ids!(rel2_x_input))
            .text()
            .parse::<f32>()
            .unwrap_or(1.0)
            .clamp(rel1_x, 1.0);
        let rel2_y = self
            .ui
            .text_input(cx, ids!(rel2_y_input))
            .text()
            .parse::<f32>()
            .unwrap_or(1.0)
            .clamp(rel1_y, 1.0);
        let comment = self.ui.text_input(cx, ids!(part_comment_input)).text();
        let parent = self
            .ui
            .text_input(cx, ids!(part_parent_input))
            .text()
            .trim()
            .to_owned();

        let before = self.capture_document_snapshot();
        if let Some(part) = self.current_parts_mut().get_mut(index) {
            part.name = name;
            part.source = source;
            part.clip = if clip_w > 0.0 && clip_h > 0.0 {
                Some(GlyfClip {
                    x: clip_x,
                    y: clip_y,
                    w: clip_w,
                    h: clip_h,
                })
            } else {
                None
            };
            part.rel1 = GlyfPoint {
                x: rel1_x,
                y: rel1_y,
            };
            part.rel2 = GlyfPoint {
                x: rel2_x,
                y: rel2_y,
            };
            part.comment = comment;
            part.parent = (!parent.is_empty()).then_some(parent);
        }
        self.mark_dirty(cx);
        self.sync_spatial(cx);

        self.set_status(cx, "Part metadata updated");
        self.record_history("Update part metadata", before);
    }

    pub(crate) fn remove_part(&mut self, cx: &mut Cx) {
        let Some(index) = self.selected_part else {
            self.set_status(cx, "Select a part first");
            return;
        };
        let before = self.capture_document_snapshot();
        let parts = self.current_parts_mut();
        if index < parts.len() {
            parts.remove(index);
        }
        self.selected_part = if parts.is_empty() {
            None
        } else {
            Some(index.min(parts.len() - 1))
        };
        self.mark_dirty(cx);
        self.sync_spatial(cx);

        self.set_status(cx, "Part deleted");
        self.record_history("Delete part", before);
    }

    pub(crate) fn move_part(&mut self, cx: &mut Cx, delta: isize) {
        let Some(index) = self.selected_part else {
            self.set_status(cx, "Select a part first");
            return;
        };
        let len = self.current_parts().len();
        if len == 0 {
            return;
        }
        let target = (index as isize + delta).clamp(0, len as isize - 1) as usize;
        if target != index {
            let before = self.capture_document_snapshot();
            self.current_parts_mut().swap(index, target);
            self.selected_part = Some(target);
            self.mark_dirty(cx);
            self.sync_spatial(cx);
            self.record_history("Move part", before);
            self.set_status(cx, "Part order updated");
        }
    }

    pub(crate) fn move_part_to(&mut self, cx: &mut Cx, front: bool) {
        let Some(index) = self.selected_part else {
            self.set_status(cx, "Select a part first");
            return;
        };
        let len = self.current_parts().len();
        if index >= len || len < 2 {
            return;
        }
        let before = self.capture_document_snapshot();
        let part = self.current_parts_mut().remove(index);
        let target = if front { 0 } else { len - 1 };
        self.current_parts_mut().insert(target, part);
        self.selected_part = Some(target);
        self.mark_dirty(cx);
        self.sync_spatial(cx);
        self.record_history(
            if front {
                "Move part to back"
            } else {
                "Move part to front"
            },
            before,
        );
        self.set_status(
            cx,
            if front {
                "Part moved to back"
            } else {
                "Part moved to front"
            },
        );
    }
}
