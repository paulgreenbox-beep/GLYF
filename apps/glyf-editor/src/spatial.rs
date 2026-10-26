use glyf_core::{GlyfAnnotation, GlyfAnnotationGeometry, GlyfClip, GlyfPart, GlyfPoint};
use makepad_widgets::*;
use std::{collections::HashMap, sync::Arc};

pub const MIN_REGION_DRAG_PX: f64 = 6.0;
pub const MIN_CLIP_DRAG_PX: f64 = 4.0;
pub const MIN_VIEW_ZOOM: f64 = 0.10;
pub const MAX_VIEW_ZOOM: f64 = 8.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CanvasTool {
    #[default]
    Select,
    Point,
    Region,
}

#[derive(Clone, Debug)]
pub enum AnnotationCanvasAction {
    Create {
        rel: GlyfPoint,
    },
    CreateRegion {
        rel1: GlyfPoint,
        rel2: GlyfPoint,
    },
    Select {
        index: Option<usize>,
    },
    Move {
        index: usize,
        rel: GlyfPoint,
        final_change: bool,
    },
    UpdateRegion {
        index: usize,
        rel1: GlyfPoint,
        rel2: GlyfPoint,
        final_change: bool,
    },
}

#[derive(Clone, Debug)]
pub struct CanvasPart {
    pub source: String,
    pub clip: Option<GlyfClip>,
    pub rel1: GlyfPoint,
    pub rel2: GlyfPoint,
}

#[derive(Clone, Debug)]
struct CanvasAnnotation {
    geometry: GlyfAnnotationGeometry,
}

#[derive(Clone, Debug)]
pub enum SpatialCanvasAction {
    Select {
        index: Option<usize>,
    },
    ContextMenu {
        pos: DVec2,
    },
    Update {
        index: usize,
        rel1: GlyfPoint,
        rel2: GlyfPoint,
        final_change: bool,
    },
}
#[derive(Clone, Debug)]
pub struct CanvasViewportAction {
    pub zoom: f64,
    pub pan: DVec2,
}

#[derive(Clone, Debug)]
pub struct CanvasHoverAction {
    pub annotation: Option<usize>,
    pub part: Option<usize>,
}

#[derive(Clone, Debug)]
pub enum CanvasEditAction {
    Annotation { index: usize },
    Part { index: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PartDragMode {
    Move,
    Resize(RegionHandle),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CanvasHit {
    Annotation {
        index: usize,
        handle: Option<RegionHandle>,
    },
    Part {
        index: usize,
        mode: PartDragMode,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionHandle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub fn resize_cursor(handle: RegionHandle) -> MouseCursor {
    match handle {
        RegionHandle::TopLeft | RegionHandle::BottomRight => MouseCursor::NwseResize,
        RegionHandle::TopRight | RegionHandle::BottomLeft => MouseCursor::NeswResize,
    }
}

/// Canonical screen -> viewport -> content transform used by both editor canvases.
#[derive(Clone, Copy, Debug)]
pub struct ViewportTransform {
    pub viewport: Rect,
    pub content_size: Option<DVec2>,
    pub zoom: f64,
    pub pan: DVec2,
}

impl ViewportTransform {
    pub fn new(viewport: Rect, content_size: Option<DVec2>, zoom: f64, pan: DVec2) -> Self {
        Self {
            viewport,
            content_size: content_size.filter(|size| size.x > 0.0 && size.y > 0.0),
            zoom: zoom.clamp(MIN_VIEW_ZOOM, MAX_VIEW_ZOOM),
            pan,
        }
    }

    pub fn fit_rect(self) -> Rect {
        let Some(content_size) = self.content_size else {
            return self.viewport;
        };
        let scale =
            (self.viewport.size.x / content_size.x).min(self.viewport.size.y / content_size.y);
        let size = content_size * scale;
        Rect {
            pos: self.viewport.pos + (self.viewport.size - size) * 0.5,
            size,
        }
    }

    pub fn clamped_pan(self) -> DVec2 {
        let Some(_content_size) = self.content_size else {
            return DVec2::default();
        };
        let size = self.fit_rect().size * self.zoom;
        let max_pan = dvec2(
            ((size.x - self.viewport.size.x) * 0.5).max(0.0),
            ((size.y - self.viewport.size.y) * 0.5).max(0.0),
        );
        dvec2(
            self.pan.x.clamp(-max_pan.x, max_pan.x),
            self.pan.y.clamp(-max_pan.y, max_pan.y),
        )
    }

    pub fn content_rect(self) -> Rect {
        if self.content_size.is_none() {
            return self.viewport;
        }
        let size = self.fit_rect().size * self.zoom;
        Rect {
            pos: self.viewport.pos + self.viewport.size * 0.5 + self.clamped_pan() - size * 0.5,
            size,
        }
    }

    pub fn screen_to_content(self, point: DVec2) -> Option<DVec2> {
        let content = self.content_rect();
        content.contains(point).then_some(point - content.pos)
    }

    pub fn zoom_around(self, anchor: DVec2, next_zoom: f64) -> Self {
        let old_content = self.content_rect();
        if !old_content.contains(anchor) || old_content.size.x <= 0.0 || old_content.size.y <= 0.0 {
            return self;
        }
        let relative = dvec2(
            (anchor.x - old_content.pos.x) / old_content.size.x,
            (anchor.y - old_content.pos.y) / old_content.size.y,
        );
        let next_zoom = next_zoom.clamp(MIN_VIEW_ZOOM, MAX_VIEW_ZOOM);
        let next_size = self.fit_rect().size * next_zoom;
        let center = self.viewport.pos + self.viewport.size * 0.5;
        let raw_pan = anchor
            - center
            - (dvec2(relative.x * next_size.x, relative.y * next_size.y) - next_size * 0.5);
        let mut next = Self::new(self.viewport, self.content_size, next_zoom, raw_pan);
        next.pan = next.clamped_pan();
        next
    }

    pub fn one_to_one_zoom(self) -> f64 {
        let Some(content_size) = self.content_size else {
            return 1.0;
        };
        let fit = self.fit_rect();
        if fit.size.x <= 0.0 || fit.size.y <= 0.0 {
            return 1.0;
        }
        (content_size.x / fit.size.x)
            .max(content_size.y / fit.size.y)
            .clamp(MIN_VIEW_ZOOM, MAX_VIEW_ZOOM)
    }
}

pub fn zoom_view_around(
    viewport: Rect,
    image_size: Option<DVec2>,
    zoom: f64,
    pan: DVec2,
    anchor: DVec2,
    next_zoom: f64,
) -> CanvasViewportAction {
    let next =
        ViewportTransform::new(viewport, image_size, zoom, pan).zoom_around(anchor, next_zoom);
    CanvasViewportAction {
        zoom: next.zoom,
        pan: next.pan,
    }
}

pub fn one_to_one_zoom(viewport: Rect, image_size: Option<DVec2>) -> f64 {
    ViewportTransform::new(viewport, image_size, 1.0, DVec2::default()).one_to_one_zoom()
}

#[derive(Clone, Copy, Debug)]
enum AnnotationDragMode {
    Point,
    RegionMove,
    RegionResize(RegionHandle),
    CreateRegion,
}

pub fn canvas_content_rect(viewport: Rect, image_size: Option<DVec2>) -> Rect {
    ViewportTransform::new(viewport, image_size, 1.0, DVec2::default()).fit_rect()
}

pub fn clamp_view_pan(viewport: Rect, image_size: Option<DVec2>, zoom: f64, pan: DVec2) -> DVec2 {
    ViewportTransform::new(viewport, image_size, zoom, pan).clamped_pan()
}

pub fn zoomed_content_rect(
    viewport: Rect,
    image_size: Option<DVec2>,
    zoom: f64,
    pan: DVec2,
) -> Rect {
    ViewportTransform::new(viewport, image_size, zoom, pan).content_rect()
}

pub fn screen_to_canvas_view(
    viewport: Rect,
    image_size: Option<DVec2>,
    zoom: f64,
    pan: DVec2,
    screen_point: DVec2,
) -> Option<DVec2> {
    ViewportTransform::new(viewport, image_size, zoom, pan).screen_to_content(screen_point)
}

pub fn screen_to_source_pixel(
    viewport: Rect,
    source_size: DVec2,
    zoom: f64,
    pan: DVec2,
    screen_point: DVec2,
) -> Option<DVec2> {
    let content = zoomed_content_rect(viewport, Some(source_size), zoom, pan);
    if !content.contains(screen_point) {
        return None;
    }
    screen_to_source_pixel_clamped(viewport, source_size, zoom, pan, screen_point)
}

pub fn screen_to_source_pixel_clamped(
    viewport: Rect,
    source_size: DVec2,
    zoom: f64,
    pan: DVec2,
    screen_point: DVec2,
) -> Option<DVec2> {
    let content = zoomed_content_rect(viewport, Some(source_size), zoom, pan);
    if content.size.x <= 0.0 || content.size.y <= 0.0 {
        return None;
    }
    let local = screen_point - content.pos;
    Some(dvec2(
        (local.x / content.size.x * source_size.x).clamp(0.0, source_size.x),
        (local.y / content.size.y * source_size.y).clamp(0.0, source_size.y),
    ))
}

pub fn source_pixel_to_screen(
    viewport: Rect,
    source_size: DVec2,
    zoom: f64,
    pan: DVec2,
    pixel: DVec2,
) -> DVec2 {
    let content = zoomed_content_rect(viewport, Some(source_size), zoom, pan);
    content.pos
        + dvec2(
            pixel.x.clamp(0.0, source_size.x) / source_size.x * content.size.x,
            pixel.y.clamp(0.0, source_size.y) / source_size.y * content.size.y,
        )
}

pub fn source_rect_to_screen(
    viewport: Rect,
    source_size: DVec2,
    zoom: f64,
    pan: DVec2,
    clip: GlyfClip,
) -> Rect {
    let start = source_pixel_to_screen(
        viewport,
        source_size,
        zoom,
        pan,
        dvec2(clip.x as f64, clip.y as f64),
    );
    let end = source_pixel_to_screen(
        viewport,
        source_size,
        zoom,
        pan,
        dvec2((clip.x + clip.w) as f64, (clip.y + clip.h) as f64),
    );
    Rect {
        pos: start,
        size: end - start,
    }
}

pub fn normalize_source_clip(a: DVec2, b: DVec2, source_size: DVec2) -> Option<GlyfClip> {
    if source_size.x <= 0.0 || source_size.y <= 0.0 {
        return None;
    }
    let min = dvec2(a.x.min(b.x), a.y.min(b.y));
    let max = dvec2(a.x.max(b.x), a.y.max(b.y));
    let x = min.x.floor().clamp(0.0, source_size.x);
    let y = min.y.floor().clamp(0.0, source_size.y);
    let end_x = max.x.ceil().clamp(0.0, source_size.x);
    let end_y = max.y.ceil().clamp(0.0, source_size.y);
    let end_x = end_x.max((x + 1.0).min(source_size.x));
    let end_y = end_y.max((y + 1.0).min(source_size.y));
    (end_x > x && end_y > y).then_some(GlyfClip {
        x: x as f32,
        y: y as f32,
        w: (end_x - x) as f32,
        h: (end_y - y) as f32,
    })
}

pub fn clamp_source_clip(clip: GlyfClip, source_size: DVec2) -> GlyfClip {
    normalize_source_clip(
        dvec2(clip.x as f64, clip.y as f64),
        dvec2((clip.x + clip.w) as f64, (clip.y + clip.h) as f64),
        source_size,
    )
    .unwrap_or(GlyfClip {
        x: 0.0,
        y: 0.0,
        w: source_size.x.max(1.0) as f32,
        h: source_size.y.max(1.0) as f32,
    })
}
#[allow(dead_code)]
pub fn screen_to_canvas(
    viewport: Rect,
    image_size: Option<DVec2>,
    screen_point: DVec2,
) -> Option<DVec2> {
    let content = canvas_content_rect(viewport, image_size);
    if content.contains(screen_point) {
        Some(screen_point - content.pos)
    } else {
        None
    }
}

pub fn canvas_to_relative(canvas_size: DVec2, canvas_point: DVec2) -> GlyfPoint {
    GlyfPoint {
        x: (canvas_point.x / canvas_size.x.max(1.0)).clamp(0.0, 1.0) as f32,
        y: (canvas_point.y / canvas_size.y.max(1.0)).clamp(0.0, 1.0) as f32,
    }
}

pub fn relative_to_canvas(canvas_size: DVec2, rel: GlyfPoint) -> DVec2 {
    dvec2(
        canvas_size.x * rel.x.clamp(0.0, 1.0) as f64,
        canvas_size.y * rel.y.clamp(0.0, 1.0) as f64,
    )
}

#[allow(dead_code)]
pub fn screen_to_relative(rect: Rect, point: DVec2) -> GlyfPoint {
    let local = point - rect.pos;
    canvas_to_relative(rect.size, local)
}

#[allow(dead_code)]
pub fn relative_to_screen(rect: Rect, rel: GlyfPoint) -> DVec2 {
    rect.pos + relative_to_canvas(rect.size, rel)
}

pub fn normalize_region(a: GlyfPoint, b: GlyfPoint) -> Option<(GlyfPoint, GlyfPoint)> {
    let rel1 = GlyfPoint {
        x: a.x.min(b.x).clamp(0.0, 1.0),
        y: a.y.min(b.y).clamp(0.0, 1.0),
    };
    let rel2 = GlyfPoint {
        x: a.x.max(b.x).clamp(0.0, 1.0),
        y: a.y.max(b.y).clamp(0.0, 1.0),
    };
    (rel2.x > rel1.x && rel2.y > rel1.y).then_some((rel1, rel2))
}

pub fn move_region(rel1: GlyfPoint, rel2: GlyfPoint, delta: GlyfPoint) -> (GlyfPoint, GlyfPoint) {
    let width = (rel2.x - rel1.x).max(0.01);
    let height = (rel2.y - rel1.y).max(0.01);
    let next1 = GlyfPoint {
        x: (rel1.x + delta.x).clamp(0.0, 1.0 - width),
        y: (rel1.y + delta.y).clamp(0.0, 1.0 - height),
    };
    (
        next1,
        GlyfPoint {
            x: next1.x + width,
            y: next1.y + height,
        },
    )
}

pub fn resize_region(
    rel1: GlyfPoint,
    rel2: GlyfPoint,
    cursor: GlyfPoint,
    handle: RegionHandle,
) -> (GlyfPoint, GlyfPoint) {
    let fixed = match handle {
        RegionHandle::TopLeft => rel2,
        RegionHandle::TopRight => GlyfPoint {
            x: rel1.x,
            y: rel2.y,
        },
        RegionHandle::BottomLeft => GlyfPoint {
            x: rel2.x,
            y: rel1.y,
        },
        RegionHandle::BottomRight => rel1,
    };
    fn separated(value: f32, fixed: f32) -> f32 {
        let value = value.clamp(0.0, 1.0);
        if (value - fixed).abs() >= 0.01 {
            value
        } else if fixed + 0.01 <= 1.0 {
            fixed + 0.01
        } else {
            fixed - 0.01
        }
    }
    let cursor = GlyfPoint {
        x: separated(cursor.x, fixed.x),
        y: separated(cursor.y, fixed.y),
    };
    normalize_region(cursor, fixed).unwrap_or((rel1, rel2))
}

pub fn image_size_from_data(bytes: &[u8]) -> Option<DVec2> {
    let image = decode_image_from_data(bytes).ok()?;
    Some(dvec2(image.width as f64, image.height as f64))
}

impl SpatialCanvas {
    pub fn set_parts(&mut self, cx: &mut Cx, parts: &[GlyfPart], selected: Option<usize>) {
        self.parts = parts
            .iter()
            .map(|part| CanvasPart {
                source: part.source.clone(),
                clip: part.clip,
                rel1: part.rel1,
                rel2: part.rel2,
            })
            .collect();
        self.selected = selected;
        self.area.redraw(cx);
    }

    pub fn set_annotations(
        &mut self,
        cx: &mut Cx,
        annotations: &[GlyfAnnotation],
        selected: Option<usize>,
    ) {
        self.annotations = annotations
            .iter()
            .map(|annotation| CanvasAnnotation {
                geometry: annotation.geometry.clone(),
            })
            .collect();
        self.selected_annotation = selected;
        self.area.redraw(cx);
    }

    /// Shows an annotation that is being composed in the editor but has not
    /// been committed to the GLYF document yet.
    pub fn set_pending_annotation(&mut self, cx: &mut Cx, annotation: Option<&GlyfAnnotation>) {
        self.pending_annotation = annotation.map(|annotation| CanvasAnnotation {
            geometry: annotation.geometry.clone(),
        });
        self.area.redraw(cx);
    }

    pub fn set_tool(&mut self, cx: &mut Cx, tool: CanvasTool) {
        if self.tool == tool {
            return;
        }
        self.tool = tool;
        self.annotation_drag_mode = None;
        self.annotation_drag_origin = None;
        self.live_region = None;
        self.area.redraw(cx);
    }

    pub fn set_view(&mut self, cx: &mut Cx, zoom: f64, pan: DVec2) {
        let zoom = zoom.clamp(MIN_VIEW_ZOOM, MAX_VIEW_ZOOM);
        if self.zoom == zoom && self.pan == pan {
            return;
        }
        self.zoom = zoom;
        self.pan = pan;
        self.area.redraw(cx);
    }

    pub fn set_image_size(&mut self, cx: &mut Cx, image_size: Option<DVec2>) {
        if self.image_size == image_size {
            return;
        }
        self.image_size = image_size;
        self.area.redraw(cx);
    }

    pub fn set_atlases(&mut self, cx: &mut Cx, images: &[(String, Arc<[u8]>)]) {
        self.textures.clear();
        for (name, bytes) in images {
            if let Ok(image) = decode_image_from_data(bytes) {
                let size = dvec2(image.width as f64, image.height as f64);
                self.textures
                    .insert(name.clone(), (image.into_new_texture(cx), size));
            }
        }
        self.area.redraw(cx);
    }

    fn annotation_hit_at(
        &self,
        content: Rect,
        point: DVec2,
        index: usize,
        handles: bool,
    ) -> Option<(usize, Option<RegionHandle>)> {
        let annotation = self.annotations.get(index)?;
        match &annotation.geometry {
            GlyfAnnotationGeometry::Point { rel } => {
                let marker = content.pos + relative_to_canvas(content.size, *rel);
                ((marker - point).length() <= 12.0).then_some((index, None))
            }
            GlyfAnnotationGeometry::Region { rel1, rel2 } => {
                let top_left = content.pos + relative_to_canvas(content.size, *rel1);
                let bottom_right = content.pos + relative_to_canvas(content.size, *rel2);
                let corners = [
                    (RegionHandle::TopLeft, top_left),
                    (RegionHandle::TopRight, dvec2(bottom_right.x, top_left.y)),
                    (RegionHandle::BottomLeft, dvec2(top_left.x, bottom_right.y)),
                    (RegionHandle::BottomRight, bottom_right),
                ];
                if handles {
                    if let Some((handle, _)) = corners
                        .iter()
                        .find(|(_, corner)| (*corner - point).length() <= 12.0)
                    {
                        return Some((index, Some(*handle)));
                    }
                }
                let inside = point.x >= top_left.x
                    && point.x <= bottom_right.x
                    && point.y >= top_left.y
                    && point.y <= bottom_right.y;
                inside.then_some((index, None))
            }
        }
    }

    fn hit_annotation(&self, content: Rect, point: DVec2) -> Option<(usize, Option<RegionHandle>)> {
        if let Some(index) = self.selected_annotation {
            if let Some(hit) = self.annotation_hit_at(content, point, index, true) {
                return Some(hit);
            }
        }
        self.annotations
            .iter()
            .enumerate()
            .rev()
            .filter(|(index, _)| Some(*index) != self.selected_annotation)
            .find_map(|(index, _)| self.annotation_hit_at(content, point, index, false))
    }

    fn part_hit_at(
        &self,
        content: Rect,
        point: DVec2,
        index: usize,
        handles: bool,
    ) -> Option<(usize, PartDragMode)> {
        let part = self.parts.get(index)?;
        let top_left = content.pos + relative_to_canvas(content.size, part.rel1);
        let bottom_right = content.pos + relative_to_canvas(content.size, part.rel2);
        let corners = [
            (RegionHandle::TopLeft, top_left),
            (RegionHandle::TopRight, dvec2(bottom_right.x, top_left.y)),
            (RegionHandle::BottomLeft, dvec2(top_left.x, bottom_right.y)),
            (RegionHandle::BottomRight, bottom_right),
        ];
        if handles {
            if let Some((handle, _)) = corners
                .iter()
                .find(|(_, corner)| (*corner - point).length() <= 12.0)
            {
                return Some((index, PartDragMode::Resize(*handle)));
            }
        }
        let inside = point.x >= top_left.x
            && point.x <= bottom_right.x
            && point.y >= top_left.y
            && point.y <= bottom_right.y;
        inside.then_some((index, PartDragMode::Move))
    }

    fn hit_part(&self, content: Rect, point: DVec2) -> Option<(usize, PartDragMode)> {
        if let Some(index) = self.selected {
            if let Some(hit) = self.part_hit_at(content, point, index, true) {
                return Some(hit);
            }
        }
        self.parts
            .iter()
            .enumerate()
            .rev()
            .filter(|(index, _)| Some(*index) != self.selected)
            .find_map(|(index, _)| self.part_hit_at(content, point, index, false))
    }

    fn hit_target(&self, content: Rect, point: DVec2) -> Option<CanvasHit> {
        let annotation = self.hit_annotation(content, point);
        let part = self.hit_part(content, point);
        if let Some((index, handle)) = annotation {
            if Some(index) == self.selected_annotation {
                return Some(CanvasHit::Annotation { index, handle });
            }
        }
        if let Some((index, mode)) = part {
            if Some(index) == self.selected {
                return Some(CanvasHit::Part { index, mode });
            }
        }
        annotation
            .map(|(index, handle)| CanvasHit::Annotation { index, handle })
            .or_else(|| part.map(|(index, mode)| CanvasHit::Part { index, mode }))
    }

    fn cursor_for_hit(&self, hit: Option<CanvasHit>) -> MouseCursor {
        match self.tool {
            CanvasTool::Point | CanvasTool::Region => MouseCursor::Crosshair,
            CanvasTool::Select => match hit {
                Some(CanvasHit::Annotation {
                    handle: Some(handle),
                    ..
                })
                | Some(CanvasHit::Part {
                    mode: PartDragMode::Resize(handle),
                    ..
                }) => resize_cursor(handle),
                Some(CanvasHit::Annotation { .. }) | Some(CanvasHit::Part { .. }) => {
                    MouseCursor::Grab
                }
                None => MouseCursor::Default,
            },
        }
    }
    fn post_annotation_action(action: AnnotationCanvasAction) {
        Cx::post_action(action);
    }

    fn post_part_action(&self, index: usize, part: &CanvasPart, final_change: bool) {
        Cx::post_action(SpatialCanvasAction::Update {
            index,
            rel1: part.rel1,
            rel2: part.rel2,
            final_change,
        });
    }

    fn draw_part_rect(
        &mut self,
        cx: &mut Cx2d,
        rect: Rect,
        part: &CanvasPart,
        selected: bool,
        hovered: bool,
    ) {
        let pos = rect.pos + relative_to_canvas(rect.size, part.rel1);
        let size = relative_to_canvas(
            rect.size,
            GlyfPoint {
                x: (part.rel2.x - part.rel1.x).max(0.01),
                y: (part.rel2.y - part.rel1.y).max(0.01),
            },
        );
        let part_rect = Rect { pos, size };
        if let Some((texture, atlas_size)) = self.textures.get(&part.source) {
            if atlas_size.x > 0.0 && atlas_size.y > 0.0 {
                self.draw_image.draw_vars.set_texture(0, texture);
                if let Some(clip) = part.clip {
                    self.draw_image.image_scale =
                        vec2(clip.w / atlas_size.x as f32, clip.h / atlas_size.y as f32);
                    self.draw_image.image_pan =
                        vec2(clip.x / atlas_size.x as f32, clip.y / atlas_size.y as f32);
                } else {
                    self.draw_image.image_scale = vec2(1.0, 1.0);
                    self.draw_image.image_pan = vec2(0.0, 0.0);
                }
                self.draw_image.opacity = if selected {
                    1.0
                } else if hovered {
                    0.9
                } else {
                    0.76
                };
                self.draw_image.draw_abs(cx, part_rect);
            }
        }
        self.draw_part.color = if selected {
            vec4(0.16, 0.72, 1.0, 0.22)
        } else if hovered {
            vec4(0.98, 0.72, 0.18, 0.14)
        } else {
            vec4(0.30, 0.48, 0.72, 0.09)
        };
        self.draw_part.draw_abs(cx, part_rect);
        self.draw_part.color = if selected {
            vec4(0.34, 0.86, 1.0, 0.98)
        } else if hovered {
            vec4(1.0, 0.80, 0.26, 0.92)
        } else {
            vec4(0.32, 0.52, 0.78, 0.58)
        };
        let t = if selected || hovered { 2.0 } else { 1.0 };
        for edge in [
            Rect {
                pos,
                size: dvec2(size.x, t),
            },
            Rect {
                pos: dvec2(pos.x, pos.y + size.y - t),
                size: dvec2(size.x, t),
            },
            Rect {
                pos,
                size: dvec2(t, size.y),
            },
            Rect {
                pos: dvec2(pos.x + size.x - t, pos.y),
                size: dvec2(t, size.y),
            },
        ] {
            self.draw_part.draw_abs(cx, edge);
        }
        if selected {
            let end = pos + size;
            for corner in [pos, dvec2(end.x, pos.y), dvec2(pos.x, end.y), end] {
                self.draw_marker.color = vec4(0.35, 0.95, 1.0, 1.0);
                self.draw_marker.draw_abs(
                    cx,
                    Rect {
                        pos: corner - dvec2(4.0, 4.0),
                        size: dvec2(8.0, 8.0),
                    },
                );
            }
        }
    }
    fn draw_region(
        &mut self,
        cx: &mut Cx2d,
        content: Rect,
        rel1: GlyfPoint,
        rel2: GlyfPoint,
        selected: bool,
        hovered: bool,
        live: bool,
        pending: bool,
    ) {
        let pos = content.pos + relative_to_canvas(content.size, rel1);
        let end = content.pos + relative_to_canvas(content.size, rel2);
        let size = dvec2((end.x - pos.x).max(2.0), (end.y - pos.y).max(2.0));
        self.draw_part.color = if pending {
            vec4(0.68, 0.48, 1.0, 0.10)
        } else if live || selected {
            vec4(0.0, 0.92, 1.0, if live { 0.42 } else { 0.32 })
        } else if hovered {
            vec4(0.98, 0.72, 0.18, 0.10)
        } else {
            vec4(0.95, 0.70, 0.18, 0.035)
        };
        self.draw_part.draw_abs(cx, Rect { pos, size });
        self.draw_part.color = if pending {
            vec4(0.80, 0.64, 1.0, 0.98)
        } else if live || selected {
            vec4(0.0, 1.0, 1.0, 1.0)
        } else if hovered {
            vec4(1.0, 0.78, 0.22, 0.92)
        } else {
            vec4(1.0, 0.78, 0.22, 0.58)
        };
        let thickness = if pending || live || selected || hovered {
            4.0
        } else {
            1.0
        };
        self.draw_segmented_edge(cx, pos, size.x, true, thickness);
        self.draw_segmented_edge(
            cx,
            dvec2(pos.x, pos.y + size.y - thickness),
            size.x,
            true,
            thickness,
        );
        self.draw_segmented_edge(cx, pos, size.y, false, thickness);
        self.draw_segmented_edge(
            cx,
            dvec2(pos.x + size.x - thickness, pos.y),
            size.y,
            false,
            thickness,
        );
        if selected || live || pending {
            let marker = if pending {
                vec4(0.86, 0.73, 1.0, 1.0)
            } else {
                vec4(0.85, 1.0, 1.0, 1.0)
            };
            for corner in [pos, dvec2(end.x, pos.y), dvec2(pos.x, end.y), end] {
                self.draw_marker.color = marker;
                self.draw_marker.draw_abs(
                    cx,
                    Rect {
                        pos: corner - dvec2(7.0, 7.0),
                        size: dvec2(14.0, 14.0),
                    },
                );
            }
        }
    }

    fn draw_segmented_edge(
        &mut self,
        cx: &mut Cx2d,
        pos: DVec2,
        length: f64,
        horizontal: bool,
        thickness: f64,
    ) {
        let dash = if thickness > 1.0 { 11.0 } else { 8.0 };
        let gap = if thickness > 1.0 { 4.0 } else { 5.0 };
        let mut offset = 0.0;
        while offset < length {
            let segment = (length - offset).min(dash);
            self.draw_part.draw_abs(
                cx,
                if horizontal {
                    Rect {
                        pos: dvec2(pos.x + offset, pos.y),
                        size: dvec2(segment, thickness),
                    }
                } else {
                    Rect {
                        pos: dvec2(pos.x, pos.y + offset),
                        size: dvec2(thickness, segment),
                    }
                },
            );
            offset += dash + gap;
        }
    }
}

#[derive(Script, ScriptHook, WidgetRef, WidgetRegister)]
pub struct SpatialCanvas {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[live]
    draw_bg: DrawColor,
    #[live]
    draw_part: DrawColor,
    #[live]
    draw_image: DrawImage,
    #[live]
    draw_marker: DrawColor,
    #[rust]
    area: Area,
    #[live(true)]
    visible: bool,
    #[rust]
    parts: Vec<CanvasPart>,
    #[rust]
    selected: Option<usize>,
    #[rust]
    tool: CanvasTool,
    #[rust]
    annotations: Vec<CanvasAnnotation>,
    #[rust]
    selected_annotation: Option<usize>,
    #[rust]
    pending_annotation: Option<CanvasAnnotation>,
    #[rust]
    image_size: Option<DVec2>,
    #[rust]
    zoom: f64,
    #[rust]
    pan: DVec2,
    #[rust]
    pan_start: DVec2,
    #[rust]
    pan_origin: DVec2,
    #[rust]
    pan_active: bool,
    #[rust]
    hovered_annotation: Option<usize>,
    #[rust]
    hovered_part: Option<usize>,
    #[rust]
    annotation_drag_mode: Option<AnnotationDragMode>,
    #[rust]
    annotation_drag_start: DVec2,
    #[rust]
    annotation_drag_origin: Option<GlyfAnnotationGeometry>,
    #[rust]
    annotation_drag_changed: bool,
    #[rust]
    live_region: Option<(GlyfPoint, GlyfPoint)>,
    #[rust]
    textures: HashMap<String, (Texture, DVec2)>,
    #[rust]
    drag_index: Option<usize>,
    #[rust]
    drag_mode: Option<PartDragMode>,
    #[rust]
    drag_changed: bool,
    #[rust]
    drag_start: DVec2,
    #[rust]
    drag_origin: Option<(GlyfPoint, GlyfPoint)>,
}

impl SpatialCanvas {
    fn view_zoom(&self) -> f64 {
        if self.zoom <= 0.0 {
            1.0
        } else {
            self.zoom.clamp(MIN_VIEW_ZOOM, MAX_VIEW_ZOOM)
        }
    }

    fn update_hover(&mut self, cx: &mut Cx, viewport: Rect, point: DVec2) {
        let content = zoomed_content_rect(viewport, self.image_size, self.view_zoom(), self.pan);
        let hit = content
            .contains(point)
            .then(|| self.hit_target(content, point))
            .flatten();
        let (hovered_annotation, hovered_part) = match hit {
            Some(CanvasHit::Annotation { index, .. }) => (Some(index), None),
            Some(CanvasHit::Part { index, .. }) => (None, Some(index)),
            None => (None, None),
        };
        if hovered_annotation != self.hovered_annotation || hovered_part != self.hovered_part {
            self.hovered_annotation = hovered_annotation;
            self.hovered_part = hovered_part;
            Cx::post_action(CanvasHoverAction {
                annotation: hovered_annotation,
                part: hovered_part,
            });
            self.area.redraw(cx);
        }
        cx.set_cursor(self.cursor_for_hit(hit));
    }

    fn post_selected_annotation_geometry(&self, final_change: bool) {
        let Some(index) = self.selected_annotation else {
            return;
        };
        let Some(annotation) = self.annotations.get(index) else {
            return;
        };
        match annotation.geometry {
            GlyfAnnotationGeometry::Point { rel } => {
                Self::post_annotation_action(AnnotationCanvasAction::Move {
                    index,
                    rel,
                    final_change,
                });
            }
            GlyfAnnotationGeometry::Region { rel1, rel2 } => {
                Self::post_annotation_action(AnnotationCanvasAction::UpdateRegion {
                    index,
                    rel1,
                    rel2,
                    final_change,
                });
            }
        }
    }

    pub fn cancel_interaction(&mut self, cx: &mut Cx) -> bool {
        let mut cancelled = false;
        if self.pan_active {
            self.pan = self.pan_origin;
            Cx::post_action(CanvasViewportAction {
                zoom: self.view_zoom(),
                pan: self.pan,
            });
            cancelled = true;
        }
        if self.annotation_drag_mode.is_some() {
            if let (Some(index), Some(origin)) = (
                self.selected_annotation,
                self.annotation_drag_origin.clone(),
            ) {
                match origin {
                    GlyfAnnotationGeometry::Point { rel } => {
                        Self::post_annotation_action(AnnotationCanvasAction::Move {
                            index,
                            rel,
                            final_change: false,
                        });
                    }
                    GlyfAnnotationGeometry::Region { rel1, rel2 } => {
                        Self::post_annotation_action(AnnotationCanvasAction::UpdateRegion {
                            index,
                            rel1,
                            rel2,
                            final_change: false,
                        });
                    }
                }
            }
            cancelled = true;
        }
        if let (Some(index), Some((rel1, rel2))) = (self.drag_index, self.drag_origin) {
            Cx::post_action(SpatialCanvasAction::Update {
                index,
                rel1,
                rel2,
                final_change: false,
            });
            cancelled = true;
        }
        self.pan_active = false;
        self.annotation_drag_mode = None;
        self.annotation_drag_origin = None;
        self.annotation_drag_changed = false;
        self.live_region = None;
        self.drag_index = None;
        self.drag_mode = None;
        self.drag_origin = None;
        self.drag_changed = false;
        if cancelled {
            cx.set_cursor(MouseCursor::Default);
            self.area.redraw(cx);
        }
        cancelled
    }

    pub fn interaction_active(&self) -> bool {
        self.pan_active || self.annotation_drag_mode.is_some() || self.drag_index.is_some()
    }
}
impl WidgetNode for SpatialCanvas {
    fn widget_uid(&self) -> WidgetUid {
        self.uid
    }

    fn walk(&mut self, _cx: &mut Cx) -> Walk {
        self.walk
    }

    fn area(&self) -> Area {
        self.area
    }

    fn redraw(&mut self, cx: &mut Cx) {
        self.area.redraw(cx);
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, cx: &mut Cx, visible: bool) {
        if self.visible != visible {
            self.visible = visible;
            self.redraw(cx);
        }
    }
}

impl Widget for SpatialCanvas {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let hit = event.hits_with_capture_overload(cx, self.area, true);
        match hit {
            Hit::FingerScroll(fs) => {
                let old_zoom = self.view_zoom();
                let factor = (1.0 + fs.scroll.y.abs() / 600.0).clamp(1.02, 1.35);
                let next_zoom = if fs.scroll.y >= 0.0 {
                    old_zoom * factor
                } else {
                    old_zoom / factor
                };
                let view = zoom_view_around(
                    fs.rect,
                    self.image_size,
                    old_zoom,
                    self.pan,
                    fs.abs,
                    next_zoom,
                );
                self.zoom = view.zoom;
                self.pan = view.pan;
                Cx::post_action(view);
                self.area.redraw(cx);
            }
            Hit::FingerDown(fe) if fe.mouse_button() == Some(MouseButton::MIDDLE) => {
                self.pan_active = true;
                self.pan_start = fe.abs;
                self.pan_origin = self.pan;
                cx.set_cursor(MouseCursor::Grabbing);
            }
            Hit::FingerDown(fe) if fe.mouse_button() == Some(MouseButton::SECONDARY) => {
                // This originates on the UI thread, so queue it on this Cx rather than
                // through the background action signal. That makes a lone right-click
                // deliver its menu action in the following Event::Actions pass.
                cx.action(SpatialCanvasAction::ContextMenu { pos: fe.abs });
            }
            Hit::FingerDown(fe) if fe.is_primary_hit() => {
                let content =
                    zoomed_content_rect(fe.rect, self.image_size, self.view_zoom(), self.pan);
                let Some(local) = screen_to_canvas_view(
                    fe.rect,
                    self.image_size,
                    self.view_zoom(),
                    self.pan,
                    fe.abs,
                ) else {
                    if self.tool == CanvasTool::Select {
                        self.selected = None;
                        self.selected_annotation = None;
                        Self::post_annotation_action(AnnotationCanvasAction::Select {
                            index: None,
                        });
                        Cx::post_action(SpatialCanvasAction::Select { index: None });
                        self.area.redraw(cx);
                    }
                    return;
                };
                let rel = canvas_to_relative(content.size, local);
                match self.tool {
                    CanvasTool::Point => {
                        Self::post_annotation_action(AnnotationCanvasAction::Create { rel });
                    }
                    CanvasTool::Region => {
                        self.annotation_drag_mode = Some(AnnotationDragMode::CreateRegion);
                        self.annotation_drag_start = local;
                        self.annotation_drag_origin = None;
                        self.annotation_drag_changed = false;
                        self.live_region = None;
                        cx.set_cursor(MouseCursor::Crosshair);
                        self.area.redraw(cx);
                    }
                    CanvasTool::Select => match self.hit_target(content, fe.abs) {
                        Some(CanvasHit::Annotation { index, handle }) => {
                            self.selected = None;
                            self.selected_annotation = Some(index);
                            Cx::post_action(SpatialCanvasAction::Select { index: None });
                            Self::post_annotation_action(AnnotationCanvasAction::Select {
                                index: Some(index),
                            });
                            self.area.redraw(cx);
                            if fe.tap_count >= 2 {
                                Cx::post_action(CanvasEditAction::Annotation { index });
                                return;
                            }
                            self.annotation_drag_start = local;
                            self.annotation_drag_mode = Some(match handle {
                                Some(handle) => AnnotationDragMode::RegionResize(handle),
                                None => match self.annotations[index].geometry {
                                    GlyfAnnotationGeometry::Point { .. } => {
                                        AnnotationDragMode::Point
                                    }
                                    GlyfAnnotationGeometry::Region { .. } => {
                                        AnnotationDragMode::RegionMove
                                    }
                                },
                            });
                            self.annotation_drag_origin =
                                Some(self.annotations[index].geometry.clone());
                            self.annotation_drag_changed = false;
                            cx.set_cursor(
                                handle.map(resize_cursor).unwrap_or(MouseCursor::Grabbing),
                            );
                        }
                        Some(CanvasHit::Part { index, mode }) => {
                            self.selected = Some(index);
                            self.selected_annotation = None;
                            Self::post_annotation_action(AnnotationCanvasAction::Select {
                                index: None,
                            });
                            Cx::post_action(SpatialCanvasAction::Select { index: Some(index) });
                            self.area.redraw(cx);
                            if fe.tap_count >= 2 {
                                Cx::post_action(CanvasEditAction::Part { index });
                                return;
                            }
                            self.drag_index = Some(index);
                            self.drag_mode = Some(mode);
                            self.drag_start = local;
                            self.drag_origin =
                                self.parts.get(index).map(|part| (part.rel1, part.rel2));
                            self.drag_changed = false;
                            cx.set_cursor(match mode {
                                PartDragMode::Move => MouseCursor::Grabbing,
                                PartDragMode::Resize(handle) => resize_cursor(handle),
                            });
                        }
                        None => {
                            self.selected = None;
                            self.selected_annotation = None;
                            Self::post_annotation_action(AnnotationCanvasAction::Select {
                                index: None,
                            });
                            Cx::post_action(SpatialCanvasAction::Select { index: None });
                            cx.set_cursor(MouseCursor::Default);
                            self.area.redraw(cx);
                        }
                    },
                }
            }
            Hit::FingerMove(fe) => {
                if self.pan_active {
                    self.pan = clamp_view_pan(
                        fe.rect,
                        self.image_size,
                        self.view_zoom(),
                        self.pan_origin + fe.abs - self.pan_start,
                    );
                    Cx::post_action(CanvasViewportAction {
                        zoom: self.view_zoom(),
                        pan: self.pan,
                    });
                    self.area.redraw(cx);
                    return;
                }
                let content =
                    zoomed_content_rect(fe.rect, self.image_size, self.view_zoom(), self.pan);
                if content.size.x <= 0.0 || content.size.y <= 0.0 {
                    return;
                }
                let clamped = dvec2(
                    fe.abs
                        .x
                        .clamp(content.pos.x, content.pos.x + content.size.x),
                    fe.abs
                        .y
                        .clamp(content.pos.y, content.pos.y + content.size.y),
                );
                let local = clamped - content.pos;
                let current = canvas_to_relative(content.size, local);
                match (
                    self.annotation_drag_mode,
                    self.annotation_drag_origin.clone(),
                ) {
                    (Some(AnnotationDragMode::CreateRegion), _) => {
                        let start = canvas_to_relative(content.size, self.annotation_drag_start);
                        self.live_region = normalize_region(start, current);
                        self.annotation_drag_changed = true;
                        self.area.redraw(cx);
                        return;
                    }
                    (
                        Some(AnnotationDragMode::Point),
                        Some(GlyfAnnotationGeometry::Point { rel: origin }),
                    ) => {
                        let start = canvas_to_relative(content.size, self.annotation_drag_start);
                        let rel = GlyfPoint {
                            x: (origin.x + current.x - start.x).clamp(0.0, 1.0),
                            y: (origin.y + current.y - start.y).clamp(0.0, 1.0),
                        };
                        if let Some(annotation) = self
                            .selected_annotation
                            .and_then(|index| self.annotations.get_mut(index))
                        {
                            annotation.geometry = GlyfAnnotationGeometry::Point { rel };
                        }
                        Self::post_annotation_action(AnnotationCanvasAction::Move {
                            index: self.selected_annotation.unwrap_or(0),
                            rel,
                            final_change: false,
                        });
                        self.annotation_drag_changed = true;
                        self.area.redraw(cx);
                        return;
                    }
                    (
                        Some(AnnotationDragMode::RegionMove),
                        Some(GlyfAnnotationGeometry::Region { rel1, rel2 }),
                    ) => {
                        let start = canvas_to_relative(content.size, self.annotation_drag_start);
                        let delta = GlyfPoint {
                            x: current.x - start.x,
                            y: current.y - start.y,
                        };
                        let (rel1, rel2) = move_region(rel1, rel2, delta);
                        if let Some(annotation) = self
                            .selected_annotation
                            .and_then(|index| self.annotations.get_mut(index))
                        {
                            annotation.geometry = GlyfAnnotationGeometry::Region { rel1, rel2 };
                        }
                        Self::post_annotation_action(AnnotationCanvasAction::UpdateRegion {
                            index: self.selected_annotation.unwrap_or(0),
                            rel1,
                            rel2,
                            final_change: false,
                        });
                        self.annotation_drag_changed = true;
                        self.area.redraw(cx);
                        return;
                    }
                    (
                        Some(AnnotationDragMode::RegionResize(handle)),
                        Some(GlyfAnnotationGeometry::Region { rel1, rel2 }),
                    ) => {
                        let (rel1, rel2) = resize_region(rel1, rel2, current, handle);
                        if let Some(annotation) = self
                            .selected_annotation
                            .and_then(|index| self.annotations.get_mut(index))
                        {
                            annotation.geometry = GlyfAnnotationGeometry::Region { rel1, rel2 };
                        }
                        Self::post_annotation_action(AnnotationCanvasAction::UpdateRegion {
                            index: self.selected_annotation.unwrap_or(0),
                            rel1,
                            rel2,
                            final_change: false,
                        });
                        self.annotation_drag_changed = true;
                        self.area.redraw(cx);
                        return;
                    }
                    _ => {}
                }
                if let (Some(index), Some(mode), Some((origin1, origin2))) =
                    (self.drag_index, self.drag_mode, self.drag_origin)
                {
                    let start = canvas_to_relative(content.size, self.drag_start);
                    let (rel1, rel2) = match mode {
                        PartDragMode::Move => move_region(
                            origin1,
                            origin2,
                            GlyfPoint {
                                x: current.x - start.x,
                                y: current.y - start.y,
                            },
                        ),
                        PartDragMode::Resize(handle) => {
                            resize_region(origin1, origin2, current, handle)
                        }
                    };
                    if let Some(part) = self.parts.get_mut(index) {
                        part.rel1 = rel1;
                        part.rel2 = rel2;
                    }
                    Cx::post_action(SpatialCanvasAction::Update {
                        index,
                        rel1,
                        rel2,
                        final_change: false,
                    });
                    self.drag_changed = true;
                    self.area.redraw(cx);
                    return;
                }
                self.update_hover(cx, fe.rect, fe.abs);
            }
            Hit::FingerUp(fe) if fe.mouse_button() == Some(MouseButton::MIDDLE) => {
                self.pan_active = false;
                self.pan = clamp_view_pan(fe.rect, self.image_size, self.view_zoom(), self.pan);
                Cx::post_action(CanvasViewportAction {
                    zoom: self.view_zoom(),
                    pan: self.pan,
                });
                self.update_hover(cx, fe.rect, fe.abs);
            }
            Hit::FingerUp(fe) if fe.is_primary_hit() => {
                if matches!(
                    self.annotation_drag_mode,
                    Some(AnnotationDragMode::CreateRegion)
                ) {
                    let content =
                        zoomed_content_rect(fe.rect, self.image_size, self.view_zoom(), self.pan);
                    let end = dvec2(
                        fe.abs
                            .x
                            .clamp(content.pos.x, content.pos.x + content.size.x),
                        fe.abs
                            .y
                            .clamp(content.pos.y, content.pos.y + content.size.y),
                    );
                    let start_screen = content.pos + self.annotation_drag_start;
                    if (end - start_screen).length() >= MIN_REGION_DRAG_PX {
                        let start = canvas_to_relative(content.size, self.annotation_drag_start);
                        let current = canvas_to_relative(content.size, end - content.pos);
                        if let Some((rel1, rel2)) = normalize_region(start, current) {
                            Self::post_annotation_action(AnnotationCanvasAction::CreateRegion {
                                rel1,
                                rel2,
                            });
                        }
                    }
                    self.live_region = None;
                } else if self.annotation_drag_changed {
                    self.post_selected_annotation_geometry(true);
                }
                if self.drag_changed {
                    if let Some(index) = self.drag_index {
                        if let Some(part) = self.parts.get(index) {
                            self.post_part_action(index, part, true);
                        }
                    }
                }
                self.drag_index = None;
                self.drag_mode = None;
                self.drag_origin = None;
                self.drag_changed = false;
                self.annotation_drag_mode = None;
                self.annotation_drag_origin = None;
                self.annotation_drag_changed = false;
                self.update_hover(cx, fe.rect, fe.abs);
                self.area.redraw(cx);
            }
            Hit::FingerHoverIn(fe) | Hit::FingerHoverOver(fe) => {
                if !self.pan_active
                    && self.drag_index.is_none()
                    && self.annotation_drag_mode.is_none()
                {
                    self.update_hover(cx, fe.rect, fe.abs);
                }
            }
            Hit::FingerHoverOut(_) => {
                if self.hovered_annotation.is_some() || self.hovered_part.is_some() {
                    self.hovered_annotation = None;
                    self.hovered_part = None;
                    Cx::post_action(CanvasHoverAction {
                        annotation: None,
                        part: None,
                    });
                    self.area.redraw(cx);
                }
                cx.set_cursor(MouseCursor::Default);
            }
            _ => {}
        }
    }
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let viewport = cx.walk_turtle_with_area(&mut self.area, walk);

        let content = zoomed_content_rect(viewport, self.image_size, self.view_zoom(), self.pan);
        self.draw_bg.draw_abs(cx, viewport);
        let parts = self.parts.clone();
        for (index, part) in parts.iter().enumerate() {
            self.draw_part_rect(
                cx,
                content,
                part,
                Some(index) == self.selected,
                Some(index) == self.hovered_part,
            );
        }
        let annotations = self.annotations.clone();
        for (index, annotation) in annotations.into_iter().enumerate() {
            match annotation.geometry {
                GlyfAnnotationGeometry::Point { rel } => {
                    let center = content.pos + relative_to_canvas(content.size, rel);
                    let selected = Some(index) == self.selected_annotation;
                    let hovered = Some(index) == self.hovered_annotation;
                    let size = if selected {
                        14.0
                    } else if hovered {
                        12.0
                    } else {
                        9.0
                    };
                    self.draw_marker.color = if selected {
                        vec4(0.25, 0.92, 1.0, 1.0)
                    } else if hovered {
                        vec4(1.0, 0.78, 0.22, 1.0)
                    } else {
                        vec4(1.0, 0.82, 0.25, 0.78)
                    };
                    self.draw_marker.draw_abs(
                        cx,
                        Rect {
                            pos: center - dvec2(size * 0.5, size * 0.5),
                            size: dvec2(size, size),
                        },
                    );
                }
                GlyfAnnotationGeometry::Region { rel1, rel2 } => self.draw_region(
                    cx,
                    content,
                    rel1,
                    rel2,
                    Some(index) == self.selected_annotation,
                    Some(index) == self.hovered_annotation,
                    false,
                    false,
                ),
            }
        }
        if let Some((rel1, rel2)) = self.live_region {
            self.draw_region(cx, content, rel1, rel2, true, false, true, false);
        }
        if let Some(annotation) = self.pending_annotation.clone() {
            match annotation.geometry {
                GlyfAnnotationGeometry::Point { rel } => {
                    let center = content.pos + relative_to_canvas(content.size, rel);
                    self.draw_marker.color = vec4(0.86, 0.73, 1.0, 1.0);
                    self.draw_marker.draw_abs(
                        cx,
                        Rect {
                            pos: center - dvec2(6.0, 6.0),
                            size: dvec2(12.0, 12.0),
                        },
                    );
                }
                GlyfAnnotationGeometry::Region { rel1, rel2 } => {
                    self.draw_region(cx, content, rel1, rel2, false, false, false, true);
                }
            }
        }
        DrawStep::done()
    }
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.widgets.SpatialCanvasBase = #(SpatialCanvas::register_widget(vm))
    mod.widgets.SpatialCanvas = mod.std.set_type_default() do mod.widgets.SpatialCanvasBase{
        width: Fill
        height: Fill
        draw_bg +: { color: #x101827 }
        draw_part +: { color: #x3f79aa55 }
        draw_image +: { opacity: 0.8 }
        draw_marker +: { color: #xffcc3fff }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_aware_transforms_respect_letterboxing() {
        let viewport = Rect {
            pos: dvec2(10.0, 20.0),
            size: dvec2(400.0, 200.0),
        };
        let content = canvas_content_rect(viewport, Some(dvec2(100.0, 100.0)));
        assert_eq!(content.pos, dvec2(110.0, 20.0));
        assert_eq!(content.size, dvec2(200.0, 200.0));
        assert_eq!(
            screen_to_canvas(viewport, Some(dvec2(100.0, 100.0)), dvec2(210.0, 120.0)),
            Some(dvec2(100.0, 100.0))
        );
        assert_eq!(
            canvas_to_relative(content.size, dvec2(100.0, 100.0)),
            GlyfPoint { x: 0.5, y: 0.5 }
        );
        assert_eq!(
            relative_to_canvas(content.size, GlyfPoint { x: 0.5, y: 0.5 }),
            dvec2(100.0, 100.0)
        );
        assert!(screen_to_canvas(viewport, Some(dvec2(100.0, 100.0)), dvec2(20.0, 30.0)).is_none());
    }

    #[test]
    fn zoom_pan_round_trip_keeps_relative_coordinates() {
        let viewport = Rect {
            pos: dvec2(20.0, 30.0),
            size: dvec2(500.0, 300.0),
        };
        let image = Some(dvec2(1000.0, 500.0));
        let zoom = 2.0;
        let pan = dvec2(80.0, -20.0);
        let content = zoomed_content_rect(viewport, image, zoom, pan);
        let rel = GlyfPoint { x: 0.37, y: 0.62 };
        let screen = content.pos + relative_to_canvas(content.size, rel);
        let local = screen_to_canvas_view(viewport, image, zoom, pan, screen).unwrap();
        assert_eq!(canvas_to_relative(content.size, local), rel);
        assert!(
            clamp_view_pan(viewport, image, zoom, dvec2(10_000.0, -10_000.0))
                .x
                .abs()
                < 300.0
        );
    }

    #[test]
    fn source_pixel_transform_round_trips_with_zoom_pan_and_letterbox() {
        let viewport = Rect {
            pos: dvec2(10.0, 20.0),
            size: dvec2(400.0, 240.0),
        };
        let source = dvec2(2048.0, 1024.0);
        let pixel = dvec2(1200.0, 300.0);
        let screen = source_pixel_to_screen(viewport, source, 1.75, dvec2(40.0, -20.0), pixel);
        let back =
            screen_to_source_pixel(viewport, source, 1.75, dvec2(40.0, -20.0), screen).unwrap();
        assert!((back.x - pixel.x).abs() < 0.01);
        assert!((back.y - pixel.y).abs() < 0.01);
    }

    #[test]
    fn zoom_around_cursor_keeps_the_same_content_point_under_cursor() {
        let viewport = Rect {
            pos: dvec2(40.0, 30.0),
            size: dvec2(500.0, 320.0),
        };
        let image = Some(dvec2(1200.0, 800.0));
        let anchor = dvec2(330.0, 190.0);
        let before = zoomed_content_rect(viewport, image, 1.0, DVec2::default());
        let before_rel = dvec2(
            (anchor.x - before.pos.x) / before.size.x,
            (anchor.y - before.pos.y) / before.size.y,
        );
        let next = zoom_view_around(viewport, image, 1.0, DVec2::default(), anchor, 2.4);
        let after = zoomed_content_rect(viewport, image, next.zoom, next.pan);
        let after_anchor =
            after.pos + dvec2(before_rel.x * after.size.x, before_rel.y * after.size.y);
        assert!((after_anchor.x - anchor.x).abs() < 0.001);
        assert!((after_anchor.y - anchor.y).abs() < 0.001);
    }

    #[test]
    fn one_to_one_zoom_maps_source_pixels_to_logical_pixels() {
        let viewport = Rect {
            pos: DVec2::default(),
            size: dvec2(400.0, 300.0),
        };
        let source = dvec2(800.0, 600.0);
        let zoom = one_to_one_zoom(viewport, Some(source));
        assert!((zoom - 2.0).abs() < 0.001);
        let a = source_pixel_to_screen(
            viewport,
            source,
            zoom,
            DVec2::default(),
            dvec2(100.0, 100.0),
        );
        let b = source_pixel_to_screen(
            viewport,
            source,
            zoom,
            DVec2::default(),
            dvec2(101.0, 101.0),
        );
        assert!((b.x - a.x - 1.0).abs() < 0.001);
        assert!((b.y - a.y - 1.0).abs() < 0.001);
    }

    #[test]
    fn fit_has_zero_pan_and_pan_is_clamped_only_when_zoomed() {
        let viewport = Rect {
            pos: dvec2(10.0, 10.0),
            size: dvec2(500.0, 300.0),
        };
        let image = Some(dvec2(800.0, 600.0));
        assert_eq!(
            clamp_view_pan(viewport, image, 1.0, dvec2(120.0, -90.0)),
            DVec2::default()
        );
        let zoomed = clamp_view_pan(viewport, image, 3.0, dvec2(10_000.0, -10_000.0));
        assert!(zoomed.x > 0.0 && zoomed.y < 0.0);
        assert!(zoomed.x < 600.0 && zoomed.y > -600.0);
    }

    #[test]
    fn source_conversion_rejects_letterbox_but_drag_variant_clamps() {
        let viewport = Rect {
            pos: DVec2::default(),
            size: dvec2(400.0, 240.0),
        };
        let source = dvec2(100.0, 100.0);
        let letterbox_point = dvec2(20.0, 120.0);
        assert!(
            screen_to_source_pixel(viewport, source, 1.0, DVec2::default(), letterbox_point)
                .is_none()
        );
        let clamped = screen_to_source_pixel_clamped(
            viewport,
            source,
            1.0,
            DVec2::default(),
            letterbox_point,
        )
        .unwrap();
        assert_eq!(clamped.x, 0.0);
        assert_eq!(clamped.y, 50.0);
    }

    #[test]
    fn region_resize_can_cross_the_fixed_corner_and_normalizes() {
        let (rel1, rel2) = resize_region(
            GlyfPoint { x: 0.2, y: 0.2 },
            GlyfPoint { x: 0.7, y: 0.7 },
            GlyfPoint { x: 0.9, y: 0.85 },
            RegionHandle::TopLeft,
        );
        assert_eq!(rel1, GlyfPoint { x: 0.7, y: 0.7 });
        assert_eq!(rel2, GlyfPoint { x: 0.9, y: 0.85 });
    }
    #[test]
    fn source_clip_normalizes_reverse_drag_and_snaps_pixels() {
        let clip = normalize_source_clip(
            dvec2(900.4, 500.2),
            dvec2(120.1, 120.8),
            dvec2(1024.0, 768.0),
        )
        .unwrap();
        assert_eq!(
            clip,
            GlyfClip {
                x: 120.0,
                y: 120.0,
                w: 781.0,
                h: 381.0
            }
        );
        let clipped = clamp_source_clip(
            GlyfClip {
                x: 900.0,
                y: 700.0,
                w: 400.0,
                h: 200.0,
            },
            dvec2(1024.0, 768.0),
        );
        assert_eq!(
            clipped,
            GlyfClip {
                x: 900.0,
                y: 700.0,
                w: 124.0,
                h: 68.0
            }
        );
    }
    #[test]
    fn region_normalization_is_direction_independent_and_rejects_zero_size() {
        let (rel1, rel2) =
            normalize_region(GlyfPoint { x: 0.8, y: 0.7 }, GlyfPoint { x: 0.2, y: 0.1 }).unwrap();
        assert_eq!(rel1, GlyfPoint { x: 0.2, y: 0.1 });
        assert_eq!(rel2, GlyfPoint { x: 0.8, y: 0.7 });
        assert!(
            normalize_region(GlyfPoint { x: 0.5, y: 0.5 }, GlyfPoint { x: 0.5, y: 0.7 }).is_none()
        );
    }

    #[test]
    fn move_region_preserves_size_and_clamps() {
        let (next1, next2) = move_region(
            GlyfPoint { x: 0.2, y: 0.3 },
            GlyfPoint { x: 0.4, y: 0.6 },
            GlyfPoint { x: 0.3, y: 0.2 },
        );
        assert_eq!(next1, GlyfPoint { x: 0.5, y: 0.5 });
        assert_eq!(next2, GlyfPoint { x: 0.7, y: 0.8 });
        let (clamped1, clamped2) = move_region(
            GlyfPoint { x: 0.8, y: 0.8 },
            GlyfPoint { x: 0.95, y: 0.95 },
            GlyfPoint { x: 0.5, y: 0.5 },
        );
        assert_eq!(clamped1, GlyfPoint { x: 0.85, y: 0.85 });
        assert_eq!(clamped2, GlyfPoint { x: 1.0, y: 1.0 });
    }

    #[test]
    fn resize_region_keeps_minimum_and_clamps() {
        let (next1, next2) = resize_region(
            GlyfPoint { x: 0.2, y: 0.2 },
            GlyfPoint { x: 0.8, y: 0.8 },
            GlyfPoint { x: 0.35, y: 0.4 },
            RegionHandle::TopLeft,
        );
        assert_eq!(next1, GlyfPoint { x: 0.35, y: 0.4 });
        assert_eq!(next2, GlyfPoint { x: 0.8, y: 0.8 });
        let (minimum1, minimum2) = resize_region(
            GlyfPoint { x: 0.2, y: 0.2 },
            GlyfPoint { x: 0.8, y: 0.8 },
            GlyfPoint { x: 0.8, y: 0.8 },
            RegionHandle::TopLeft,
        );
        assert!(minimum2.x - minimum1.x >= 0.009);
        assert!(minimum2.y - minimum1.y >= 0.009);
        assert!(minimum1.x >= 0.0 && minimum2.x <= 1.0);
    }

    #[test]
    fn legacy_rect_helpers_still_clamp() {
        let rect = Rect {
            pos: dvec2(10.0, 20.0),
            size: dvec2(200.0, 100.0),
        };
        let rel = screen_to_relative(rect, dvec2(110.0, 70.0));
        assert_eq!(rel, GlyfPoint { x: 0.5, y: 0.5 });
        assert_eq!(relative_to_screen(rect, rel), dvec2(110.0, 70.0));
        assert_eq!(
            screen_to_relative(rect, dvec2(-100.0, 999.0)),
            GlyfPoint { x: 0.0, y: 1.0 }
        );
    }
}
