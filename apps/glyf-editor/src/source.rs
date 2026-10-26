use crate::spatial::{
    CanvasViewportAction, MAX_VIEW_ZOOM, MIN_CLIP_DRAG_PX, MIN_VIEW_ZOOM, RegionHandle,
    clamp_source_clip, clamp_view_pan, normalize_source_clip, resize_cursor,
    screen_to_source_pixel, screen_to_source_pixel_clamped, source_rect_to_screen,
    zoom_view_around,
};
use glyf_core::GlyfClip;
use makepad_widgets::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SourceTool {
    #[default]
    Select,
    Clip,
}

#[derive(Clone, Debug)]
pub enum SourceCanvasAction {
    ClipChanged { clip: GlyfClip, final_change: bool },
    ClipSelected { clip: GlyfClip },
    ClipCancelled { clip: Option<GlyfClip> },
    Viewport(CanvasViewportAction),
}

#[derive(Clone, Copy, Debug)]
enum ClipDrag {
    Create {
        start: DVec2,
        screen_start: DVec2,
        original: Option<GlyfClip>,
    },
    Move {
        start: DVec2,
        origin: GlyfClip,
    },
    Resize {
        origin: GlyfClip,
        handle: RegionHandle,
    },
}

fn move_clip(origin: GlyfClip, delta: DVec2, source_size: DVec2) -> GlyfClip {
    let x = (origin.x as f64 + delta.x).clamp(0.0, (source_size.x - origin.w as f64).max(0.0));
    let y = (origin.y as f64 + delta.y).clamp(0.0, (source_size.y - origin.h as f64).max(0.0));
    GlyfClip {
        x: x.round() as f32,
        y: y.round() as f32,
        ..origin
    }
}

fn resize_clip(
    origin: GlyfClip,
    cursor: DVec2,
    handle: RegionHandle,
    source_size: DVec2,
) -> GlyfClip {
    let left = origin.x as f64;
    let top = origin.y as f64;
    let right = left + origin.w as f64;
    let bottom = top + origin.h as f64;
    let (x1, y1, x2, y2) = match handle {
        RegionHandle::TopLeft => (cursor.x, cursor.y, right, bottom),
        RegionHandle::TopRight => (left, cursor.y, cursor.x, bottom),
        RegionHandle::BottomLeft => (cursor.x, top, right, cursor.y),
        RegionHandle::BottomRight => (left, top, cursor.x, cursor.y),
    };
    normalize_source_clip(dvec2(x1, y1), dvec2(x2, y2), source_size).unwrap_or(origin)
}

#[derive(Script, ScriptHook, WidgetRef, WidgetRegister)]
pub struct SourceCanvas {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[live]
    draw_bg: DrawQuad,
    #[live]
    draw_image: DrawImage,
    #[live]
    draw_overlay: DrawQuad,
    #[live]
    draw_marker: DrawQuad,
    #[rust]
    area: Area,
    #[live(true)]
    visible: bool,
    #[rust]
    texture: Option<Texture>,
    #[rust]
    image_size: Option<DVec2>,
    #[rust]
    image_name: String,
    #[rust]
    clip: Option<GlyfClip>,
    #[rust]
    tool: SourceTool,
    #[rust]
    zoom: f64,
    #[rust]
    pan: DVec2,
    #[rust]
    pan_active: bool,
    #[rust]
    pan_start: DVec2,
    #[rust]
    pan_origin: DVec2,
    #[rust]
    drag: Option<ClipDrag>,
    #[rust]
    drag_changed: bool,
    #[rust]
    hovered_clip: bool,
}

impl SourceCanvas {
    pub fn set_source(&mut self, cx: &mut Cx, name: &str, bytes: &[u8]) {
        self.image_name = name.to_owned();
        if let Ok(image) = decode_image_from_data(bytes) {
            self.image_size = Some(dvec2(image.width as f64, image.height as f64));
            self.texture = Some(image.into_new_texture(cx));
        } else {
            self.image_size = None;
            self.texture = None;
        }
        self.clip = None;
        self.zoom = 1.0;
        self.pan = DVec2::default();
        self.area.redraw(cx);
    }

    pub fn set_clip(&mut self, cx: &mut Cx, clip: Option<GlyfClip>) {
        let clip = clip.map(|clip| {
            self.image_size
                .map(|size| clamp_source_clip(clip, size))
                .unwrap_or(clip)
        });
        if self.clip == clip {
            return;
        }
        self.clip = clip;
        self.area.redraw(cx);
    }

    pub fn set_tool(&mut self, cx: &mut Cx, tool: SourceTool) {
        if self.tool == tool {
            return;
        }
        self.tool = tool;
        self.cancel_interaction(cx);
        self.area.redraw(cx);
    }

    pub fn drag_active(&self) -> bool {
        self.drag.is_some()
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

    fn view_zoom(&self) -> f64 {
        if self.zoom <= 0.0 {
            1.0
        } else {
            self.zoom.clamp(MIN_VIEW_ZOOM, MAX_VIEW_ZOOM)
        }
    }

    fn post(action: SourceCanvasAction) {
        Cx::post_action(action);
    }

    pub fn interaction_active(&self) -> bool {
        self.pan_active || self.drag.is_some()
    }

    pub fn cancel_interaction(&mut self, cx: &mut Cx) -> bool {
        let mut cancelled = self.pan_active;
        self.pan_active = false;
        if let Some(drag) = self.drag.take() {
            self.clip = match drag {
                ClipDrag::Create { original, .. } => original,
                ClipDrag::Move { origin, .. } | ClipDrag::Resize { origin, .. } => Some(origin),
            };
            Self::post(SourceCanvasAction::ClipCancelled { clip: self.clip });
            cancelled = true;
        }
        self.drag_changed = false;
        if cancelled {
            self.hovered_clip = false;
            cx.set_cursor(MouseCursor::Default);
            self.area.redraw(cx);
        }
        cancelled
    }

    fn update_hover(&mut self, cx: &mut Cx, viewport: Rect, point: DVec2) {
        let hit = self.hit_clip(viewport, point);
        let hovered = hit.is_some();
        if hovered != self.hovered_clip {
            self.hovered_clip = hovered;
            self.area.redraw(cx);
        }
        let cursor = match (self.tool, hit) {
            (SourceTool::Clip, Some((Some(handle), _))) => resize_cursor(handle),
            (SourceTool::Clip, Some((None, _))) => MouseCursor::Grab,
            (SourceTool::Clip, None) => MouseCursor::Crosshair,
            (SourceTool::Select, Some(_)) => MouseCursor::Hand,
            (SourceTool::Select, None) => MouseCursor::Default,
        };
        cx.set_cursor(cursor);
    }

    fn hit_clip(&self, viewport: Rect, point: DVec2) -> Option<(Option<RegionHandle>, GlyfClip)> {
        let clip = self.clip?;
        let size = self.image_size?;
        let rect = source_rect_to_screen(viewport, size, self.view_zoom(), self.pan, clip);
        let corners = [
            (RegionHandle::TopLeft, rect.pos),
            (
                RegionHandle::TopRight,
                dvec2(rect.pos.x + rect.size.x, rect.pos.y),
            ),
            (
                RegionHandle::BottomLeft,
                dvec2(rect.pos.x, rect.pos.y + rect.size.y),
            ),
            (RegionHandle::BottomRight, rect.pos + rect.size),
        ];
        let handle = corners
            .iter()
            .find(|(_, corner)| (*corner - point).length() <= 12.0)
            .map(|(handle, _)| *handle);
        if let Some(handle) = handle {
            Some((Some(handle), clip))
        } else if rect.contains(point) {
            Some((None, clip))
        } else {
            None
        }
    }

    fn draw_clip(&mut self, cx: &mut Cx2d, viewport: Rect, clip: GlyfClip, hovered: bool) {
        let rect = source_rect_to_screen(
            viewport,
            self.image_size.unwrap_or(viewport.size),
            self.view_zoom(),
            self.pan,
            clip,
        );
        self.draw_overlay.draw_vars.set_uniform(
            cx,
            live_id!(color),
            if hovered {
                &[1.0, 0.72, 0.18, 0.16]
            } else {
                &[0.20, 0.86, 1.0, 0.12]
            },
        );
        self.draw_overlay.draw_abs(cx, rect);
        self.draw_overlay.draw_vars.set_uniform(
            cx,
            live_id!(color),
            if hovered {
                &[1.0, 0.78, 0.22, 1.0]
            } else {
                &[0.35, 0.92, 1.0, 0.98]
            },
        );
        let t = 2.0;
        self.draw_overlay.draw_abs(
            cx,
            Rect {
                pos: rect.pos,
                size: dvec2(rect.size.x, t),
            },
        );
        self.draw_overlay.draw_abs(
            cx,
            Rect {
                pos: dvec2(rect.pos.x, rect.pos.y + rect.size.y - t),
                size: dvec2(rect.size.x, t),
            },
        );
        self.draw_overlay.draw_abs(
            cx,
            Rect {
                pos: rect.pos,
                size: dvec2(t, rect.size.y),
            },
        );
        self.draw_overlay.draw_abs(
            cx,
            Rect {
                pos: dvec2(rect.pos.x + rect.size.x - t, rect.pos.y),
                size: dvec2(t, rect.size.y),
            },
        );
        for corner in [
            rect.pos,
            dvec2(rect.pos.x + rect.size.x, rect.pos.y),
            dvec2(rect.pos.x, rect.pos.y + rect.size.y),
            rect.pos + rect.size,
        ] {
            self.draw_marker
                .draw_vars
                .set_uniform(cx, live_id!(color), &[0.35, 0.95, 1.0, 1.0]);
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

impl WidgetNode for SourceCanvas {
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

impl Widget for SourceCanvas {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event.hits_with_capture_overload(cx, self.area, true) {
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
                Self::post(SourceCanvasAction::Viewport(view));
                self.area.redraw(cx);
            }
            Hit::FingerDown(fe) if fe.mouse_button() == Some(MouseButton::MIDDLE) => {
                self.pan_active = true;
                self.pan_start = fe.abs;
                self.pan_origin = self.pan;
                cx.set_cursor(MouseCursor::Grabbing);
            }
            Hit::FingerDown(fe) if fe.is_primary_hit() => {
                let Some(size) = self.image_size else { return };
                let Some(pixel) =
                    screen_to_source_pixel(fe.rect, size, self.view_zoom(), self.pan, fe.abs)
                else {
                    return;
                };
                if self.tool == SourceTool::Clip {
                    self.drag_changed = false;
                    if let Some((handle, clip)) = self.hit_clip(fe.rect, fe.abs) {
                        self.drag = Some(match handle {
                            Some(handle) => ClipDrag::Resize {
                                origin: clip,
                                handle,
                            },
                            None => ClipDrag::Move {
                                start: pixel,
                                origin: clip,
                            },
                        });
                        cx.set_cursor(handle.map(resize_cursor).unwrap_or(MouseCursor::Grabbing));
                    } else {
                        self.drag = Some(ClipDrag::Create {
                            start: pixel,
                            screen_start: fe.abs,
                            original: self.clip,
                        });
                        self.clip = normalize_source_clip(pixel, pixel, size);
                        cx.set_cursor(MouseCursor::Crosshair);
                    }
                    self.area.redraw(cx);
                } else if let Some((_handle, clip)) = self.hit_clip(fe.rect, fe.abs) {
                    Self::post(SourceCanvasAction::ClipSelected { clip });
                }
            }
            Hit::FingerMove(fe) => {
                if self.pan_active {
                    let Some(size) = self.image_size else { return };
                    self.pan = clamp_view_pan(
                        fe.rect,
                        Some(size),
                        self.view_zoom(),
                        self.pan_origin + fe.abs - self.pan_start,
                    );
                    Self::post(SourceCanvasAction::Viewport(CanvasViewportAction {
                        zoom: self.view_zoom(),
                        pan: self.pan,
                    }));
                    self.area.redraw(cx);
                    return;
                }
                let Some(drag) = self.drag else {
                    self.update_hover(cx, fe.rect, fe.abs);
                    return;
                };
                let Some(size) = self.image_size else { return };
                let Some(pixel) = screen_to_source_pixel_clamped(
                    fe.rect,
                    size,
                    self.view_zoom(),
                    self.pan,
                    fe.abs,
                ) else {
                    return;
                };
                let next = match drag {
                    ClipDrag::Create { start, .. } => normalize_source_clip(start, pixel, size),
                    ClipDrag::Move { start, origin } => {
                        Some(move_clip(origin, pixel - start, size))
                    }
                    ClipDrag::Resize { origin, handle } => {
                        Some(resize_clip(origin, pixel, handle, size))
                    }
                };
                if let Some(clip) = next {
                    self.clip = Some(clip);
                    self.drag_changed = true;
                    Self::post(SourceCanvasAction::ClipChanged {
                        clip,
                        final_change: false,
                    });
                    self.area.redraw(cx);
                }
            }
            Hit::FingerUp(fe) if fe.mouse_button() == Some(MouseButton::MIDDLE) => {
                self.pan_active = false;
                self.pan = clamp_view_pan(fe.rect, self.image_size, self.view_zoom(), self.pan);
                Self::post(SourceCanvasAction::Viewport(CanvasViewportAction {
                    zoom: self.view_zoom(),
                    pan: self.pan,
                }));
                self.update_hover(cx, fe.rect, fe.abs);
            }
            Hit::FingerUp(fe) if fe.is_primary_hit() => {
                if let Some(drag) = self.drag.take() {
                    let large_enough = match drag {
                        ClipDrag::Create { screen_start, .. } => {
                            (fe.abs - screen_start).length() >= MIN_CLIP_DRAG_PX
                        }
                        _ => true,
                    };
                    if large_enough {
                        if let Some(clip) = self.clip {
                            if self.drag_changed {
                                Self::post(SourceCanvasAction::ClipChanged {
                                    clip,
                                    final_change: true,
                                });
                            }
                            Self::post(SourceCanvasAction::ClipSelected { clip });
                        }
                    } else {
                        self.clip = match drag {
                            ClipDrag::Create { original, .. } => original,
                            ClipDrag::Move { origin, .. } | ClipDrag::Resize { origin, .. } => {
                                Some(origin)
                            }
                        };
                        Self::post(SourceCanvasAction::ClipCancelled { clip: self.clip });
                    }
                }
                self.drag_changed = false;
                self.update_hover(cx, fe.rect, fe.abs);
                self.area.redraw(cx);
            }
            Hit::FingerHoverIn(fe) | Hit::FingerHoverOver(fe) => {
                if !self.interaction_active() {
                    self.update_hover(cx, fe.rect, fe.abs);
                }
            }
            Hit::FingerHoverOut(_) => {
                self.hovered_clip = false;
                cx.set_cursor(MouseCursor::Default);
                self.area.redraw(cx);
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let viewport = cx.walk_turtle(walk);
        let Some(size) = self.image_size else {
            self.draw_bg.draw_abs(cx, viewport);
            self.area = self.draw_bg.area();
            return DrawStep::done();
        };
        self.draw_bg.draw_abs(cx, viewport);
        let content =
            crate::spatial::zoomed_content_rect(viewport, Some(size), self.view_zoom(), self.pan);
        if let Some(texture) = &self.texture {
            self.draw_image.draw_vars.set_texture(0, texture);
            self.draw_image.image_scale = vec2(1.0, 1.0);
            self.draw_image.image_pan = vec2(0.0, 0.0);
            self.draw_image.opacity = 1.0;
            self.draw_image.draw_abs(cx, content);
        }
        if let Some(clip) = self.clip {
            self.draw_clip(cx, viewport, clip, self.hovered_clip);
        }
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}
script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    mod.widgets.SourceCanvasBase = #(SourceCanvas::register_widget(vm))
    mod.widgets.SourceCanvas = mod.std.set_type_default() do mod.widgets.SourceCanvasBase{
        width: Fill
        height: Fill
        draw_bg +: { color: #x0b1220 }
        draw_overlay +: { color: #x3b82f655 }
        draw_marker +: { color: #x58a6d8ff }
        draw_image +: { opacity: 1.0 }
    }
}
