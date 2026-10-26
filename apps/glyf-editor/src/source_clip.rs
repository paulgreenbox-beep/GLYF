//! Source-image clipping workflow and transient clip drafts.

use super::*;

impl App {
    pub(crate) fn source_popup_position(&self, cx: &Cx) -> DVec2 {
        let viewport = self.source_canvas_rect(cx);
        let popup_width = 350.0;
        let popup_height = 190.0;
        let selected_clip = self
            .pending_clip
            .as_ref()
            .map(|pending| pending.clip)
            .or(self.source_clip);
        let marker = match (self.selected_source_size(), selected_clip) {
            (Some(size), Some(clip)) => {
                let rect = source_rect_to_screen(
                    viewport,
                    size,
                    self.source_zoom_value(),
                    self.source_pan,
                    clip,
                );
                rect.pos + rect.size
            }
            _ => viewport.pos + viewport.size * 0.5,
        };
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

    pub(crate) fn show_clip_popup_at(
        &mut self,
        cx: &mut Cx,
        title: &str,
        name: &str,
        comment: &str,
        pos: DVec2,
    ) {
        if let Some(mut popup) = self
            .ui
            .widget(cx, ids!(clip_popup))
            .borrow_mut::<ClipPopup>()
        {
            let russian = self.settings.language == Language::Russian;
            let creating = title.starts_with("Create");
            popup.set_language(cx, self.settings.language);
            let localized_title = if russian {
                match title {
                    "Edit part metadata" => "Изменить данные части",
                    "Edit source clip" => "Изменить фрагмент источника",
                    "Create part from source clip" => "Создать часть из фрагмента",
                    _ => title,
                }
            } else {
                title
            };
            popup.set_title(cx, localized_title);
            popup.set_commit_text(
                cx,
                if creating {
                    if russian { "Создать часть" } else { "Create Part" }
                } else {
                    if russian { "Применить" } else { "Apply" }
                },
            );
            popup.set_name(cx, name);
            popup.set_comment(cx, comment);
            popup.set_pos(cx, pos);
            popup.show(cx);
            popup.focus_name(cx);
        }
    }

    pub(crate) fn show_clip_popup(&mut self, cx: &mut Cx, title: &str, name: &str, comment: &str) {
        let pos = self.source_popup_position(cx);
        self.show_clip_popup_at(cx, title, name, comment, pos);
    }

    pub(crate) fn hide_clip_popup(&mut self, cx: &mut Cx) {
        if let Some(mut popup) = self
            .ui
            .widget(cx, ids!(clip_popup))
            .borrow_mut::<ClipPopup>()
        {
            popup.hide(cx);
        }
    }

    pub(crate) fn begin_new_clip(&mut self, cx: &mut Cx, clip: GlyfClip) {
        if self.pending_clip.is_some() {
            return;
        }
        let Some(source_index) = self.selected else {
            return;
        };
        let name = unique_part_name(
            self.current_parts(),
            &format!("part_{}", self.current_parts().len() + 1),
        );
        self.begin_gesture_history();
        self.source_clip = Some(clip);
        self.pending_clip = Some(PendingClip {
            part_index: None,
            source_index,
            original_clip: None,
            clip,
            original_name: name.clone(),
            original_comment: String::new(),
            draft_name: name.clone(),
            draft_comment: String::new(),
        });
        self.show_clip_popup(cx, "Create part from source clip", &name, "");
        self.set_status(cx, "Source clip selected — enter part name and comment");
    }

    pub(crate) fn begin_edit_clip(&mut self, cx: &mut Cx, index: usize) {
        let Some(part) = self.current_parts().get(index).cloned() else {
            self.set_status(cx, "Select a part with a source clip first");
            return;
        };
        let Some(clip) = part.clip else {
            self.set_status(cx, "Selected part has no source clip");
            return;
        };
        let Some(source_index) = self
            .images
            .iter()
            .position(|image| image.name == part.source)
        else {
            self.set_status(cx, "Part source image is missing");
            return;
        };
        self.begin_gesture_history();
        self.selected = Some(source_index);
        self.selected_part = Some(index);
        self.source_mode = true;
        self.source_tool = SourceTool::Clip;
        self.source_zoom = 1.0;
        self.source_pan = DVec2::default();
        self.source_clip = Some(clip);
        self.pending_clip = Some(PendingClip {
            part_index: Some(index),
            source_index,
            original_clip: Some(clip),
            clip,
            original_name: part.name.clone(),
            original_comment: part.comment.clone(),
            draft_name: part.name.clone(),
            draft_comment: part.comment.clone(),
        });
        self.load_preview(cx);
        self.refresh(cx);
        self.show_clip_popup(cx, "Edit source clip", &part.name, &part.comment);
        self.set_status(cx, "Edit Clip — move or resize, then confirm");
    }

    pub(crate) fn set_source_clip_from_accessibility(&mut self, cx: &mut Cx, value: &str) {
        let values = value
            .split(|ch: char| ch == ',' || ch == ';' || ch.is_ascii_whitespace())
            .filter(|part| !part.is_empty())
            .filter_map(|part| part.parse::<f32>().ok())
            .collect::<Vec<_>>();
        if values.len() != 4 || values.iter().any(|value| !value.is_finite()) {
            self.set_status(cx, "Source clip expects x,y,w,h pixel values");
            return;
        }
        let Some(size) = self.selected_source_size() else {
            self.set_status(cx, "Select an image before setting a source clip");
            return;
        };
        let clip = clamp_source_clip(
            GlyfClip {
                x: values[0],
                y: values[1],
                w: values[2].max(1.0),
                h: values[3].max(1.0),
            },
            size,
        );
        self.source_mode = true;
        self.source_tool = SourceTool::Clip;
        self.source_clip = Some(clip);
        if let Some(pending) = self.pending_clip.as_mut() {
            pending.clip = clip;
            self.sync_source_canvas(cx);
        } else {
            self.begin_new_clip(cx, clip);
        }
        self.set_status(cx, "Semantic source clip geometry applied");
    }

    pub(crate) fn handle_source_canvas_action(&mut self, cx: &mut Cx, action: SourceCanvasAction) {
        match action {
            SourceCanvasAction::Viewport(view) => {
                self.source_zoom = view.zoom.clamp(MIN_VIEW_ZOOM, MAX_VIEW_ZOOM);
                self.source_pan = view.pan;
                self.sync_source_canvas(cx);
            }
            SourceCanvasAction::ClipChanged { clip, final_change } => {
                self.source_clip = Some(clip);
                if let Some(pending) = self.pending_clip.as_mut() {
                    pending.clip = clip;
                }
                self.sync_source_canvas(cx);
                if final_change {
                    self.set_status(cx, "Source clip geometry updated");
                }
            }
            SourceCanvasAction::ClipSelected { clip } => {
                self.source_clip = Some(clip);
                if let Some(pending) = self.pending_clip.as_mut() {
                    pending.clip = clip;
                    self.sync_source_canvas(cx);
                } else if self.source_tool == SourceTool::Clip {
                    let editable = self.selected_part.and_then(|index| {
                        let source_name = self
                            .selected
                            .and_then(|source| self.images.get(source))
                            .map(|image| image.name.as_str())?;
                        self.current_parts()
                            .get(index)
                            .filter(|part| part.source == source_name && part.clip.is_some())
                            .map(|_| index)
                    });
                    if let Some(index) = editable {
                        self.begin_edit_clip(cx, index);
                        self.source_clip = Some(clip);
                        if let Some(pending) = self.pending_clip.as_mut() {
                            pending.clip = clip;
                        }
                        self.sync_source_canvas(cx);
                    } else {
                        self.begin_new_clip(cx, clip);
                    }
                } else {
                    self.set_status(cx, "Source clip selected");
                    self.sync_source_canvas(cx);
                }
            }
            SourceCanvasAction::ClipCancelled { clip } => {
                self.source_clip = clip;
                if let (Some(pending), Some(clip)) = (self.pending_clip.as_mut(), clip) {
                    pending.clip = clip;
                }
                self.sync_source_canvas(cx);
                self.set_status(cx, "Source clip gesture cancelled");
            }
        }
    }

    pub(crate) fn commit_clip(&mut self, cx: &mut Cx) {
        let Some(pending) = self.pending_clip.take() else {
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
        let name = if popup_name.is_empty() || popup_name == pending.original_name {
            pending.draft_name.clone()
        } else {
            popup_name
        };
        let comment = if popup_comment.is_empty() || popup_comment == pending.original_comment {
            pending.draft_comment.clone()
        } else {
            popup_comment
        };
        if name.is_empty() {
            self.pending_clip = Some(pending);
            self.set_status(cx, "Part name is required");
            return;
        }
        let edited_index = pending.part_index;
        if self
            .current_parts()
            .iter()
            .enumerate()
            .any(|(index, part)| Some(index) != edited_index && part.name == name)
        {
            self.pending_clip = Some(pending);
            self.set_status(cx, "Part names must be unique within the group");
            return;
        }
        if let Some(index) = edited_index {
            if let Some(part) = self.current_parts_mut().get_mut(index) {
                part.clip = Some(pending.clip);
                part.name = name;
                part.comment = comment;
            }
            self.source_clip = Some(pending.clip);
            self.hide_clip_popup(cx);
            self.source_mode = false;
            self.mark_dirty(cx);
            self.refresh(cx);
            self.finish_gesture_history("Update source clip and part metadata");
            self.set_status(cx, "Source clip and part metadata updated");
            return;
        }
        let Some(image) = self.images.get(pending.source_index).cloned() else {
            self.set_status(cx, "Source image is missing");
            return;
        };
        let Some(size) = image.pixel_size else {
            self.set_status(cx, "Source image dimensions are unavailable");
            return;
        };
        let normalized_ratio =
            (pending.clip.w as f64 * size.y) / (pending.clip.h as f64 * size.x).max(1.0);
        let width = (0.34_f64).min(0.18 * normalized_ratio.max(0.1));
        let height = (width / normalized_ratio.max(0.1)).min(0.34);
        let rel1 = GlyfPoint {
            x: (0.5 - width * 0.5).max(0.0) as f32,
            y: (0.5 - height * 0.5).max(0.0) as f32,
        };
        let rel2 = GlyfPoint {
            x: (rel1.x as f64 + width).min(1.0) as f32,
            y: (rel1.y as f64 + height).min(1.0) as f32,
        };
        let index = self.current_parts().len();
        self.current_parts_mut().push(GlyfPart {
            name,
            source: image.name,
            clip: Some(pending.clip),
            rel1,
            rel2,
            comment,
            parent: None,
        });
        self.selected_part = Some(index);
        self.selected = Some(pending.source_index);
        self.source_clip = Some(pending.clip);
        self.hide_clip_popup(cx);
        self.source_mode = false;
        self.mark_dirty(cx);
        self.refresh(cx);
        self.finish_gesture_history("Create part from source clip");
        self.set_status(cx, "Part created from source clip");
    }

    pub(crate) fn cancel_clip(&mut self, cx: &mut Cx) {
        let pending = self.pending_clip.take();
        self.source_clip = pending.as_ref().and_then(|pending| pending.original_clip);
        self.hide_clip_popup(cx);
        self.refresh(cx);
        self.set_status(cx, "Source clip cancelled");
    }
}
