use makepad_widgets::*;

#[derive(Script, Widget)]
pub struct ClipPopup {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[rust]
    draw_list: Option<DrawList2d>,
    #[live]
    draw_bg: DrawQuad,
    #[rust]
    opened: bool,
    #[rust]
    popup_pos: Vec2d,
}

impl ScriptHook for ClipPopup {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        self.draw_list = Some(DrawList2d::script_new(vm));
    }

    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        vm.with_cx_mut(|cx| {
            if let Some(draw_list) = &self.draw_list {
                draw_list.redraw(cx);
            }
        });
    }
}

impl Widget for ClipPopup {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.opened {
            self.view.handle_event(cx, event, scope);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, _walk: Walk) -> DrawStep {
        let draw_list = self.draw_list.as_mut().expect("clip popup draw list");
        draw_list.begin_overlay_reuse(cx);
        let size = cx.current_pass_size();
        cx.begin_root_turtle(size, self.view.layout);
        self.draw_bg.begin(cx, self.view.walk, self.view.layout);
        if self.opened {
            let content_walk = self.view.walk(cx).with_abs_pos(self.popup_pos);
            self.view.draw_walk_all(cx, scope, content_walk);
        }
        self.draw_bg.end(cx);
        cx.end_pass_sized_turtle();
        draw_list.end(cx);
        DrawStep::done()
    }
}

impl ClipPopup {
    /// Applies the popup's built-in labels for the selected interface language.
    /// Call before setting task-specific title and commit text.
    pub fn set_language(&mut self, cx: &mut Cx, language: crate::desktop::Language) {
        let (title, name, comment, cancel, create) = match language {
            crate::desktop::Language::Russian => (
                "Создать фрагмент",
                "Название фрагмента",
                "Комментарий",
                "Отмена",
                "Создать фрагмент",
            ),
            crate::desktop::Language::English => (
                "Create part from clip",
                "Part name",
                "Comment",
                "Cancel",
                "Create Part",
            ),
        };
        self.set_title(cx, title);
        self.set_commit_text(cx, create);
        self.view
            .text_input(cx, ids!(content.name_input))
            .set_empty_text(cx, name.to_owned());
        self.view
            .text_input(cx, ids!(content.comment_input))
            .set_empty_text(cx, comment.to_owned());
        self.view
            .button(cx, ids!(content.actions.cancel))
            .set_text(cx, cancel);
    }

    pub fn set_title(&mut self, cx: &mut Cx, value: &str) {
        self.view.label(cx, ids!(content.title)).set_text(cx, value);
    }

    pub fn set_commit_text(&mut self, cx: &mut Cx, value: &str) {
        self.view
            .button(cx, ids!(content.actions.commit))
            .set_text(cx, value);
    }
    pub fn set_name(&mut self, cx: &mut Cx, value: &str) {
        self.view
            .text_input(cx, ids!(content.name_input))
            .set_text(cx, value);
    }

    pub fn name(&self, cx: &mut Cx) -> String {
        self.view.text_input(cx, ids!(content.name_input)).text()
    }

    pub fn set_comment(&mut self, cx: &mut Cx, value: &str) {
        self.view
            .text_input(cx, ids!(content.comment_input))
            .set_text(cx, value);
    }

    pub fn comment(&self, cx: &mut Cx) -> String {
        self.view.text_input(cx, ids!(content.comment_input)).text()
    }

    pub fn focus_name(&mut self, cx: &mut Cx) {
        self.view
            .text_input(cx, ids!(content.name_input))
            .set_key_focus(cx);
    }

    pub fn focus_comment(&mut self, cx: &mut Cx) {
        self.view
            .text_input(cx, ids!(content.comment_input))
            .set_key_focus(cx);
    }

    pub fn name_returned(&mut self, cx: &mut Cx, actions: &Actions) -> bool {
        self.view
            .text_input(cx, ids!(content.name_input))
            .returned(actions)
            .is_some()
    }

    pub fn comment_returned(&mut self, cx: &mut Cx, actions: &Actions) -> bool {
        self.view
            .text_input(cx, ids!(content.comment_input))
            .returned(actions)
            .is_some()
    }

    pub fn commit_clicked(&mut self, cx: &mut Cx, actions: &Actions) -> bool {
        self.view
            .button(cx, ids!(content.actions.commit))
            .clicked(actions)
    }

    pub fn cancel_clicked(&mut self, cx: &mut Cx, actions: &Actions) -> bool {
        self.view
            .button(cx, ids!(content.actions.cancel))
            .clicked(actions)
    }

    pub fn set_pos(&mut self, cx: &mut Cx, pos: Vec2d) {
        self.popup_pos = pos;
        self.redraw(cx);
    }

    pub fn show(&mut self, cx: &mut Cx) {
        self.opened = true;
        if let Some(draw_list) = &self.draw_list {
            draw_list.redraw(cx);
        }
        self.redraw(cx);
    }

    pub fn hide(&mut self, cx: &mut Cx) {
        self.opened = false;
        if let Some(draw_list) = &self.draw_list {
            draw_list.redraw(cx);
        }
        self.redraw(cx);
    }
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    mod.widgets.ClipPopupBase = #(ClipPopup::register_widget(vm))
    mod.widgets.ClipPopup = mod.std.set_type_default() do mod.widgets.ClipPopupBase{
        width: Fill
        height: Fill
        flow: Overlay
        align: Align {x: 0.0 y: 0.0}
        draw_bg +: { color: #x00000000 }
        content := RoundedView {
            width: 350
            height: Fit
            flow: Down
            spacing: 8
            padding: 12
            draw_bg +: {
                color: #x172033
                border_size: 1.0
                border_color: #x58a6d855
                radius: 8.0
            }
            title := Label {
                width: Fill
                height: Fit
                text: "Create part from clip"
                draw_text.color: #xe2e8f0
            }
            name_input := TextInput {
                width: Fill
                height: 36
                empty_text: "Part name"
            }
            comment_input := TextInput {
                width: Fill
                height: 60
                empty_text: "Comment"
                return_key_type: Done
            }
            actions := View {
                width: Fill
                height: Fit
                flow: Right
                spacing: 8
                align: Align {x: 1.0 y: 0.5}
                cancel := Button { text: "Cancel" }
                commit := Button { text: "Create Part" }
            }
        }
    }
}
