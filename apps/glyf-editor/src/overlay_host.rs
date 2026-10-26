use makepad_widgets::*;

/// A reusable editor-local overlay owner.
///
/// The host stays a normal root-level Makepad `View`: this gives it ordinary
/// sibling z-order and durable widget hit areas on Windows. Dropdowns and
/// dialogs live in `content`, are anchored with `walk.abs_pos`, and never
/// participate in the editor's normal layout flow.
#[derive(Clone, Debug, Default)]
pub enum OverlayHostAction {
    Dismissed,
    #[default]
    None,
}

#[derive(Script, ScriptHook, Widget)]
pub struct OverlayHost {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[rust]
    opened: bool,
    #[rust]
    dismiss_on_outside: bool,
    #[rust]
    popup_pos: Vec2d,
    #[rust]
    popup_size: Vec2d,
    #[rust]
    chain_mode: bool,
}

impl Widget for OverlayHost {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.opened {
            return;
        }

        self.view.handle_event(cx, event, scope);

        if !self.dismiss_on_outside {
            return;
        }
        let popup_rect = Rect {
            pos: self.popup_pos,
            size: self.popup_size,
        };
        let outside_click = match event {
            Event::MouseDown(pointer) => !popup_rect.contains(pointer.abs),
            _ => false,
        };
        if outside_click {
            self.hide(cx);
            Cx::post_action(OverlayHostAction::Dismissed);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.opened {
            return DrawStep::done();
        }
        if self.chain_mode {
            self.view.draw_walk_all(cx, scope, walk);
        } else {
            let mut overlay_walk = self.view.walk(cx);
            overlay_walk.abs_pos = Some(self.popup_pos);
            overlay_walk.width = Size::Fixed(self.popup_size.x);
            overlay_walk.height = Size::Fixed(self.popup_size.y);
            self.view.draw_walk_all(cx, scope, overlay_walk);
        }
        cx.block_scrolling_except_within(self.view.area());
        DrawStep::done()
    }
}

impl OverlayHost {
    pub fn show(&mut self, cx: &mut Cx, pos: Vec2d, size: Vec2d, dismiss_on_outside: bool) {
        self.popup_pos = pos;
        self.popup_size = size;
        self.dismiss_on_outside = dismiss_on_outside;
        self.chain_mode = false;
        self.opened = true;
        self.redraw(cx);
    }

    /// Shows the full overlay surface so a root menu and its descendants can
    /// have independent absolute positions without participating in layout.
    pub fn show_chain(&mut self, cx: &mut Cx, dismiss_on_outside: bool) {
        self.dismiss_on_outside = dismiss_on_outside;
        self.chain_mode = true;
        self.opened = true;
        self.redraw(cx);
    }

    pub fn hide(&mut self, cx: &mut Cx) {
        if !self.opened {
            return;
        }
        self.opened = false;
        self.dismiss_on_outside = false;
        self.chain_mode = false;
        self.redraw(cx);
        cx.unblock_scrolling();
    }
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.widgets.GlyfOverlayHostBase = #(OverlayHost::register_widget(vm))
    mod.widgets.GlyfOverlayHost = mod.std.set_type_default() do mod.widgets.GlyfOverlayHostBase{
        width: Fill
        height: Fill
        flow: Overlay
        align: Align {x: 0.0 y: 0.0}

        // A transparent real Fill child keeps pointer hit-testing in the
        // regular widget tree rather than relying on a DrawList2d portal.
        bg_view := View {
            width: Fill
            height: Fill
            show_bg: true
            draw_bg +: {
                color: #x00000000
                pixel: fn() {
                    return self.color
                }
            }
        }

    }
}
