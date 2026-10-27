//! Image import/preview and GLYF/GLYP persistence commands.

use super::*;

impl App {
    fn choose_file(&mut self, cx: &mut Cx, kind: FileDialogKind) -> Option<PathBuf> {
        match file_dialog::choose_for_language(kind, self.settings.language) {
            Ok(Some(path)) => Some(path),
            Ok(None) => {
                let status = match kind {
                    FileDialogKind::OpenProject => "Open cancelled",
                    FileDialogKind::SaveProject => "Save As cancelled",
                    FileDialogKind::OpenImage => "Add image cancelled",
                };
                self.set_status(cx, status);
                None
            }
            Err(error) => {
                let prefix = if self.settings.language == Language::Russian {
                    "Не удалось открыть системный файловый диалог"
                } else {
                    "Could not open the system file dialog"
                };
                self.set_status(cx, format!("{prefix}: {error}"));
                None
            }
        }
    }

    pub(crate) fn open_project_action(&mut self, cx: &mut Cx) {
        let Some(path) = self.choose_file(cx, FileDialogKind::OpenProject) else {
            return;
        };
        self.project_path = path.display().to_string();
        self.open_project(cx);
    }

    pub(crate) fn save_project_as(&mut self, cx: &mut Cx) {
        let Some(path) = self.choose_file(cx, FileDialogKind::SaveProject) else {
            return;
        };
        self.project_path = path.display().to_string();
        self.save_project(cx);
    }

    pub(crate) fn save_project_command(&mut self, cx: &mut Cx) {
        if self.project_path.trim().is_empty() || self.project_path.starts_with("Untitled") {
            self.save_project_as(cx);
        } else {
            self.save_project(cx);
        }
    }

    pub(crate) fn add_image_action(&mut self, cx: &mut Cx) {
        let typed = self.ui.text_input(cx, ids!(image_path_input)).text();
        if typed.trim().is_empty() {
            if let Some(path) = self.choose_file(cx, FileDialogKind::OpenImage) {
                self.ui
                    .text_input(cx, ids!(image_path_input))
                    .set_text(cx, &path.display().to_string());
            } else {
                return;
            }
        }
        self.add_image(cx);
    }

    pub(crate) fn add_image(&mut self, cx: &mut Cx) {
        let raw_path = self.ui.text_input(cx, ids!(image_path_input)).text();
        let path = PathBuf::from(raw_path.trim());
        if raw_path.trim().is_empty() {
            self.set_status(cx, "Choose an image path first");
            return;
        }
        if !supported_image(&path) {
            self.set_status(cx, "Supported images: PNG, JPG, JPEG, WEBP, BMP, ICO");
            return;
        }
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                self.set_status(cx, format!("Cannot read image: {error}"));
                return;
            }
        };
        let source = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("image.bin")
            .to_owned();
        let fallback_name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .unwrap_or("image");
        let name = unique_name(&self.images, fallback_name);
        let before = self.capture_document_snapshot();
        let pixel_size = image_size_from_data(&bytes);
        self.images.push(EditorImage {
            name,
            source,
            comment: String::new(),
            bytes: bytes.into(),
            pixel_size,
        });
        self.selected = Some(self.images.len() - 1);
        self.atlases_dirty = true;
        self.mark_dirty(cx);
        self.load_preview(cx);
        self.refresh(cx);
        self.set_status(cx, "Image added; enter a name and comment, then Update");
        self.record_history("Add image", before);
    }

    pub(crate) fn load_preview(&mut self, cx: &mut Cx) {
        let Some(selected) = self.selected else {
            return;
        };
        let Some(image) = self.images.get(selected) else {
            return;
        };
        let image_name = image.name.clone();
        let bytes = image.bytes.clone();
        if let Err(error) = self
            .ui
            .image(cx, ids!(preview_image))
            .load_image_from_data(cx, &bytes)
        {
            self.set_status(cx, format!("Preview decode failed: {error:?}"));
        }
        if let Some(mut canvas) = self
            .ui
            .widget(cx, ids!(source_canvas))
            .borrow_mut::<SourceCanvas>()
        {
            canvas.set_source(cx, &image_name, &bytes);
        }
    }
    pub(crate) fn remove_image(&mut self, cx: &mut Cx) {
        let Some(selected) = self.selected else {
            self.set_status(cx, "Select an image to remove");
            return;
        };
        if let Some(image) = self.images.get(selected) {
            if self
                .current_parts()
                .iter()
                .any(|part| part.source == image.name)
            {
                self.set_status(cx, "Remove parts that use this resource first");
                return;
            }
        }
        let before = self.capture_document_snapshot();
        self.images.remove(selected);
        self.atlases_dirty = true;
        self.mark_dirty(cx);
        self.selected = if self.images.is_empty() {
            None
        } else {
            Some(selected.min(self.images.len() - 1))
        };
        self.load_preview(cx);
        self.refresh(cx);
        self.set_status(cx, "Image removed");
        self.record_history("Remove image", before);
    }

    pub(crate) fn update_image(&mut self, cx: &mut Cx) {
        let Some(selected) = self.selected else {
            self.set_status(cx, "Select an image to update");
            return;
        };
        let old_name = self.images[selected].name.clone();
        let name = self
            .ui
            .text_input(cx, ids!(name_input))
            .text()
            .trim()
            .to_owned();
        if name.is_empty() {
            self.set_status(cx, "Image name cannot be empty");
            return;
        }
        if self
            .images
            .iter()
            .enumerate()
            .any(|(index, image)| index != selected && image.name == name)
        {
            self.set_status(cx, "Image names must be unique");
            return;
        }
        let comment = self.ui.text_input(cx, ids!(comment_input)).text();
        let before = self.capture_document_snapshot();
        if let Some(image) = self.images.get_mut(selected) {
            image.name = name.clone();
            image.comment = comment;
        }
        for part in self.current_parts_mut() {
            if part.source == old_name {
                part.source = name.clone();
            }
        }
        // Comments do not change image textures or their lookup keys.
        self.atlases_dirty |= name != old_name;
        self.mark_dirty(cx);
        self.refresh(cx);
        self.set_status(cx, "Image metadata updated");
        self.record_history("Update image metadata", before);
    }

    pub(crate) fn save_project(&mut self, cx: &mut Cx) {
        if self.has_pending_work() || self.canvas_interaction_active(cx) {
            self.set_status(cx, "Finish or cancel the current edit before saving");
            return;
        }
        let glyf_path = with_extension(&self.project_path, "glyf");
        let glyp_path = glyf_path.with_extension("glyp");
        let document = GlyfDocument {
            version: 1,
            images: self
                .images
                .iter()
                .map(|image| GlyfImage {
                    name: image.name.clone(),
                    source: image.source.clone(),
                    comment: image.comment.clone(),
                })
                .collect(),
            groups: self.groups.clone(),
            annotations: self.annotations.clone(),
        };
        if let Err(error) = serialize_file(&document, &glyf_path) {
            self.set_status(cx, format!("Save failed: {error}"));
            return;
        }
        let mut writer = GlypWriter::new();
        let manifest_name = glyf_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("project.glyf")
            .to_owned();
        let manifest_bytes = match fs::read(&glyf_path) {
            Ok(bytes) => bytes,
            Err(error) => {
                self.set_status(cx, format!("Pack failed: {error}"));
                return;
            }
        };
        if let Err(error) = writer.add_file(manifest_name, manifest_bytes) {
            self.set_status(cx, format!("Pack failed: {error}"));
            return;
        }
        for image in &self.images {
            if let Err(error) = writer.add_file(image.source.clone(), image.bytes.to_vec()) {
                self.set_status(cx, format!("Pack failed: {error}"));
                return;
            }
        }
        if let Err(error) = writer.write(&glyp_path) {
            self.set_status(cx, format!("Pack failed: {error}"));
            return;
        }
        self.project_path = glyf_path.display().to_string();
        self.set_dirty(cx, false);
        self.settings.add_recent(&glyf_path);
        let _ = self.settings.save();
        self.sync_shell_labels(cx);
        self.set_status(
            cx,
            format!("Saved {} and {}", glyf_path.display(), glyp_path.display()),
        );
    }

    pub(crate) fn open_project(&mut self, cx: &mut Cx) {
        let raw_path = self.project_path.trim().to_owned();
        if raw_path.is_empty() {
            self.set_status(cx, "Enter a .glyf path first");
            return;
        }
        if !self.confirm_replace(cx, ReplaceIntent::Open) {
            return;
        }
        let glyf_path = with_extension(&raw_path, "glyf");
        let glyp_path = glyf_path.with_extension("glyp");
        let document = match parse_file(&glyf_path) {
            Ok(document) => document,
            Err(error) => {
                self.set_status(cx, format!("Open failed: {error}"));
                return;
            }
        };
        let mut reader = match GlypReader::open(&glyp_path) {
            Ok(reader) => reader,
            Err(error) => {
                self.set_status(cx, format!("Open failed: {error}"));
                return;
            }
        };
        let mut loaded = Vec::with_capacity(document.images.len());
        let groups = document.groups.clone();
        let annotations = document.annotations.clone();
        for image in document.images {
            let bytes = match reader.read_file(&image.source) {
                Ok(bytes) => bytes,
                Err(error) => {
                    self.set_status(cx, format!("Missing resource '{}': {error}", image.source));
                    return;
                }
            };
            let pixel_size = image_size_from_data(&bytes);
            loaded.push(EditorImage {
                name: image.name,
                source: image.source,
                comment: image.comment,
                bytes: bytes.into(),
                pixel_size,
            });
        }
        let reuse_blank = self.tabs.len() == 1
            && self.images.is_empty()
            && self.groups.is_empty()
            && self.annotations.is_empty()
            && !self.dirty;
        if !reuse_blank {
            self.sync_active_tab_snapshot();
            self.undo_stack.clear();
            self.redo_stack.clear();
            self.gesture_history_before = None;
            let tab_id = self.allocate_tab_id();
            self.tabs.push(DocumentTab {
                id: tab_id,
                title: tab_title(&glyf_path.display().to_string(), self.tabs.len()),
                path: glyf_path.display().to_string(),
                dirty: false,
                snapshot: DocumentSnapshot::default(),
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
            });
            self.active_tab = self.tabs.len() - 1;
        }
        self.images = loaded;
        self.groups = groups;
        self.annotations = annotations;
        self.pending_annotation = None;
        self.pending_clip = None;
        self.pending_part_popup = None;
        self.annotation_draft.clear();
        self.source_mode = false;
        self.source_tool = SourceTool::Select;
        self.source_clip = None;
        self.source_zoom = 1.0;
        self.source_pan = DVec2::default();
        self.canvas_zoom = self.settings.default_zoom;
        self.canvas_pan = DVec2::default();
        self.selected_annotation = (!self.annotations.is_empty()).then_some(0);
        self.hide_annotation_popup(cx);
        self.hide_clip_popup(cx);
        self.selected_part = (!self.current_parts().is_empty()).then_some(0);
        self.selected = if self.images.is_empty() {
            None
        } else {
            Some(0)
        };
        self.project_path = glyf_path.display().to_string();
        self.atlases_dirty = true;
        self.load_preview(cx);
        self.set_dirty(cx, false);
        self.settings.add_recent(&glyf_path);
        let _ = self.settings.save();
        self.refresh(cx);
        self.sync_shell_labels(cx);
        self.set_status(cx, format!("Opened {}", glyf_path.display()));
    }
}
