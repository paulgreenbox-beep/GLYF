use makepad_widgets::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnnotationPopupPointerTarget {
    Comment,
    Cancel,
    Commit,
}

#[derive(Script, Widget)]
pub struct AnnotationPopup {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[rust]
    opened: bool,
    #[rust]
    popup_pos: Vec2d,
    #[rust]
    focus_on_next_draw: bool,
}

impl ScriptHook for AnnotationPopup {}

impl Widget for AnnotationPopup {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.opened {
            return;
        }
        // This popup deliberately stays in the parent UI pass instead of a
        // DrawList2d portal. That gives its TextInput the same hit-test and
        // keyboard-focus route as the rest of the desktop UI on Windows.
        self.view
            .widget(cx, ids!(content))
            .handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, _walk: Walk) -> DrawStep {
        if !self.opened {
            return DrawStep::done();
        }
        // The editor's preview is already an Overlay. Draw the compact popup
        // in that normal pass at its semantic anchor; no extra portal pass is
        // needed and the content keeps a stable interactive Area.
        let content = self.view.widget(cx, ids!(content));
        let content_walk = content.walk(cx).with_abs_pos(self.popup_pos);
        content.draw_walk_all(cx, scope, content_walk);
        if self.focus_on_next_draw {
            self.view
                .text_input(cx, ids!(content.comment_input))
                .set_key_focus(cx);
            self.focus_on_next_draw = false;
        }
        cx.block_scrolling_except_within(content.area());
        DrawStep::done()
    }
}

impl AnnotationPopup {
    /// Applies the popup's built-in labels for the selected interface language.
    pub fn set_language(&mut self, cx: &mut Cx, language: crate::desktop::Language) {
        let (title, prompt, cancel, add) = match language {
            crate::desktop::Language::Russian => (
                "Добавить аннотацию",
                "Что здесь находится?",
                "Отмена",
                "Добавить",
            ),
            crate::desktop::Language::English => {
                ("Add annotation", "What is here?", "Cancel", "Add")
            }
        };
        self.set_title(cx, title);
        self.view
            .text_input(cx, ids!(content.comment_input))
            .set_empty_text(cx, prompt.to_owned());
        self.view
            .button(cx, ids!(content.actions.cancel))
            .set_text(cx, cancel);
        self.view
            .button(cx, ids!(content.actions.commit))
            .set_text(cx, add);
    }

    pub fn set_title(&mut self, cx: &mut Cx, title: &str) {
        self.view.label(cx, ids!(content.title)).set_text(cx, title);
    }

    pub fn set_commit_text(&mut self, cx: &mut Cx, value: &str) {
        self.view
            .button(cx, ids!(content.actions.commit))
            .set_text(cx, value);
    }

    pub fn set_comment_text(&mut self, cx: &mut Cx, text: &str) {
        self.view
            .text_input(cx, ids!(content.comment_input))
            .set_text(cx, text);
    }

    pub fn comment_text(&self, cx: &mut Cx) -> String {
        self.view.text_input(cx, ids!(content.comment_input)).text()
    }

    pub fn focus_comment(&mut self, cx: &mut Cx) {
        self.view
            .text_input(cx, ids!(content.comment_input))
            .set_key_focus(cx);
        self.focus_on_next_draw = true;
        self.redraw(cx);
    }

    pub fn pointer_target(&self, cx: &mut Cx, abs: Vec2d) -> Option<AnnotationPopupPointerTarget> {
        if !self.opened {
            return None;
        }
        if self
            .view
            .text_input(cx, ids!(content.comment_input))
            .area()
            .rect(cx)
            .contains(abs)
        {
            return Some(AnnotationPopupPointerTarget::Comment);
        }
        if self
            .view
            .button(cx, ids!(content.actions.cancel))
            .area()
            .rect(cx)
            .contains(abs)
        {
            return Some(AnnotationPopupPointerTarget::Cancel);
        }
        if self
            .view
            .button(cx, ids!(content.actions.commit))
            .area()
            .rect(cx)
            .contains(abs)
        {
            return Some(AnnotationPopupPointerTarget::Commit);
        }
        None
    }

    pub fn commit_clicked(&mut self, cx: &mut Cx, actions: &Actions) -> bool {
        self.view
            .button(cx, ids!(content.actions.commit))
            .clicked(actions)
    }

    pub fn comment_returned(&mut self, cx: &mut Cx, actions: &Actions) -> bool {
        self.view
            .text_input(cx, ids!(content.comment_input))
            .returned(actions)
            .is_some()
    }

    pub fn comment_escaped(&mut self, cx: &mut Cx, actions: &Actions) -> bool {
        self.view
            .text_input(cx, ids!(content.comment_input))
            .escaped(actions)
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
        self.focus_on_next_draw = true;
        self.redraw(cx);
    }

    pub fn hide(&mut self, cx: &mut Cx) {
        if !self.opened {
            return;
        }
        self.opened = false;
        self.focus_on_next_draw = false;
        self.redraw(cx);
        cx.revert_key_focus();
        cx.unblock_scrolling();
    }
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.widgets.AnnotationPopupBase = #(AnnotationPopup::register_widget(vm))
    mod.widgets.AnnotationPopup = mod.std.set_type_default() do mod.widgets.AnnotationPopupBase{
        width: Fill
        height: Fill
        flow: Overlay
        align: Align {x: 0.0 y: 0.0}

        content := RoundedView {
            width: 310
            height: Fit
            flow: Down
            spacing: 8
            padding: 10
            draw_bg +: {
                color: #x172033
                border_size: 1.0
                border_color: #x58a6d855
                radius: 8.0
            }

            title := Label {
                width: Fill
                height: Fit
                text: "Add annotation"
                draw_text.color: #xe2e8f0
            }

            comment_input := TextInput {
                width: Fill
                height: 44
                empty_text: "What is here?"
                return_key_type: Done
            }

            actions := View {
                width: Fill
                height: Fit
                flow: Right
                spacing: 8
                align: Align {x: 1.0 y: 0.5}
                cancel := Button { text: "Cancel" }
                commit := Button { text: "Add" }
            }
        }
    }
}
