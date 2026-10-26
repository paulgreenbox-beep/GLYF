//! A thin document-tab adapter around Makepad's scrollable `TabBar`.
//!
//! The application owns document state and stable IDs. This widget only maps
//! that state to visual keys and maps Makepad tab events back to those keys.

use makepad_widgets::widget::WidgetActionData;
use makepad_widgets::*;

/// One visual document tab supplied by the editor shell.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentTabFrame {
    /// Ephemeral renderer key mapped from an application-owned document ID.
    pub key: LiveId,
    /// Filename shown in the strip. Dirty state is deliberately separate.
    pub title: String,
    /// Full document path, retained for tooltip and accessibility consumers.
    pub full_path: String,
    /// Whether the document has unsaved changes.
    pub dirty: bool,
    /// Whether this is the selected document.
    pub active: bool,
}

impl DocumentTabFrame {
    /// Full path for a native tooltip or semantic node.
    pub fn tooltip_text(&self) -> &str {
        if self.full_path.is_empty() {
            &self.title
        } else {
            &self.full_path
        }
    }

    /// Accessible name that keeps the filename, save state, and path explicit.
    pub fn accessibility_name(&self) -> String {
        self.accessibility_name_for(crate::desktop::Language::English)
    }

    /// Accessible name translated for the current interface language.
    pub fn accessibility_name_for(&self, language: crate::desktop::Language) -> String {
        if self.dirty {
            let state = match language {
                crate::desktop::Language::Russian => "есть несохранённые изменения",
                crate::desktop::Language::English => "unsaved",
            };
            format!("{}, {}, {}", self.title, state, self.tooltip_text())
        } else {
            format!("{}, {}", self.title, self.tooltip_text())
        }
    }

    fn display_label(&self) -> String {
        format!(
            "{}{}",
            truncate_tab_title(&self.title, MAX_TAB_TITLE_CHARS),
            dirty_marker(self.dirty)
        )
    }
}

/// Keep each tab compact so the TabBar's native horizontal scrolling remains
/// useful even when a document has an unusually long filename.
const MAX_TAB_TITLE_CHARS: usize = 28;

fn truncate_tab_title(title: &str, max_chars: usize) -> String {
    let char_count = title.chars().count();
    if char_count <= max_chars {
        return title.to_owned();
    }
    if max_chars == 0 {
        return String::new();
    }

    let visible_chars = max_chars.saturating_sub(1);
    let mut truncated: String = title.chars().take(visible_chars).collect();
    truncated.push('…');
    truncated
}

fn dirty_marker(dirty: bool) -> &'static str {
    if dirty { "    ●" } else { "" }
}

/// Actions emitted by the visual tab strip.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum DocumentTabStripAction {
    #[default]
    None,
    Activate(LiveId),
    Close(LiveId),
}

/// A browser/editor-style document tab row with built-in Makepad scrolling.
#[derive(Script, ScriptHook, Widget)]
pub struct DocumentTabStrip {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[redraw]
    #[live]
    tab_bar: TabBar,

    #[action_data]
    #[rust]
    action_data: WidgetActionData,
    #[rust]
    frame: Vec<DocumentTabFrame>,
}

impl DocumentTabStrip {
    /// Replaces the visual projection and redraws the scrollable strip.
    pub fn set_tabs(&mut self, cx: &mut Cx, frame: Vec<DocumentTabFrame>) {
        self.frame = frame;
        self.tab_bar.redraw(cx);
    }

    /// Returns the actual visible rectangle of a document tab for accessibility.
    pub fn tab_rect(&self, cx: &Cx, key: LiveId) -> Option<Rect> {
        self.tab_bar.tab_rect(cx, key)
    }

    /// Returns the deliberately generous close target rendered by CloseableTab.
    /// TabBar does not expose the private TabCloseButton area, so the wrapper
    /// owns the stable hit geometry that it also reports to accessibility.
    pub fn tab_close_rect(&self, cx: &Cx, key: LiveId) -> Option<Rect> {
        self.tab_rect(cx, key).map(Self::close_rect_for_tab)
    }

    /// Returns the complete tab-bar viewport rectangle.
    pub fn bar_rect(&self, cx: &Cx) -> Rect {
        self.tab_bar.bar_rect(cx)
    }

    fn close_rect_for_tab(tab_rect: Rect) -> Rect {
        Rect {
            pos: dvec2(tab_rect.pos.x + 4.0, tab_rect.pos.y + 3.0),
            size: dvec2(
                tab_rect.size.x.min(20.0).max(1.0),
                (tab_rect.size.y - 6.0).max(1.0),
            ),
        }
    }

    fn close_key_at(&self, cx: &Cx, abs: DVec2) -> Option<LiveId> {
        self.frame.iter().find_map(|tab| {
            self.tab_close_rect(cx, tab.key)
                .filter(|rect| rect.contains(abs))
                .map(|_| tab.key)
        })
    }

    fn tab_key_at(&self, cx: &Cx, abs: DVec2) -> Option<LiveId> {
        self.frame.iter().find_map(|tab| {
            self.tab_rect(cx, tab.key)
                .filter(|rect| rect.contains(abs))
                .map(|_| tab.key)
        })
    }

    fn emit(&self, cx: &mut Cx, action: DocumentTabStripAction) {
        cx.widget_action_with_data(&self.action_data, self.uid, action);
    }
}

impl Widget for DocumentTabStrip {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // The stock TabCloseButton's DrawQuad is intentionally private to Tab.
        // Routing the documented close rectangle here makes mouse interaction
        // and the semantic node agree, while ordinary tab selection still uses
        // the scrollable Makepad TabBar below.
        if let Event::MouseDown(pointer) = event {
            if let Some(key) = self.close_key_at(cx, pointer.abs) {
                self.emit(cx, DocumentTabStripAction::Close(key));
                return;
            }
            // Keep mouse selection dependable when another native input (for
            // example an IME-backed TextInput) held key focus immediately
            // before the user returns to the document strip.  Returning here
            // also prevents TabBar's private close glyph from interpreting a
            // normal tab click as a second, competing close action. Wheel
            // scrolling and hover events still flow through TabBar below.
            if let Some(key) = self.tab_key_at(cx, pointer.abs) {
                self.emit(cx, DocumentTabStripAction::Activate(key));
                return;
            }
        }
        for action in cx.capture_actions(|cx| self.tab_bar.handle_event(cx, event, scope)) {
            match action.as_widget_action().cast() {
                TabBarAction::TabWasPressed(key) if self.frame.iter().any(|tab| tab.key == key) => {
                    self.emit(cx, DocumentTabStripAction::Activate(key));
                }
                TabBarAction::TabCloseWasPressed(key)
                    if self.frame.iter().any(|tab| tab.key == key) =>
                {
                    self.emit(cx, DocumentTabStripAction::Close(key));
                }
                TabBarAction::TabWasPressed(_)
                | TabBarAction::TabCloseWasPressed(_)
                | TabBarAction::ShouldTabStartDrag(_)
                | TabBarAction::None => {}
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let active_index = self.frame.iter().position(|tab| tab.active);
        self.tab_bar.begin(cx, active_index, walk);
        for tab in &self.frame {
            self.tab_bar
                .draw_tab(cx, tab.key, &tab.display_label(), id!(CloseableTab));
        }
        self.tab_bar.end(cx);
        DrawStep::done()
    }
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.widgets.DocumentTabStripBase = #(DocumentTabStrip::register_widget(vm))
    mod.widgets.DocumentTabStrip = mod.std.set_type_default() do mod.widgets.DocumentTabStripBase {
        width: Fill
        height: 38
        tab_bar: TabBarFlat {
            height: 38
            // The upstream glyph is 10 px wide, which is unpleasantly tiny in
            // a desktop document strip.  Retain the native TabBar event path,
            // but make the close target a compact, reliably clickable 20×30 px.
            CloseableTab +: {
                padding: Inset{left: 6 right: 8 top: 0 bottom: 0}
                icon_walk: Walk{width: 14 height: 14}
                draw_icon +: {
                    svg: crate_resource("makepad_widgets:resources/icons/icon_file.svg")
                    color: theme.color_label_inner
                }
                close_button +: {
                    width: 20
                    height: 30
                    margin: Inset{left: 0 right: 4 top: 0 bottom: 0}
                }
            }
            draw_bg +: {
                color: theme.color_bg_even
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_titles_are_truncated_without_splitting_unicode() {
        assert_eq!(truncate_tab_title("short.glyf", 28), "short.glyf");
        assert_eq!(
            truncate_tab_title("Очень-длинное🪨имя-документа.glyf", 12),
            "Очень-длинн…"
        );
        assert_eq!(truncate_tab_title("long", 1), "…");
        assert_eq!(truncate_tab_title("long", 0), "");
    }

    #[test]
    fn dirty_marker_is_independent_of_the_title() {
        let mut tab = DocumentTabFrame {
            key: id!(test_tab),
            title: "scene.glyf".to_owned(),
            full_path: "C:/work/scene.glyf".to_owned(),
            dirty: false,
            active: false,
        };
        let clean = tab.display_label();
        assert!(!clean.contains('●'));
        assert_eq!(tab.title, "scene.glyf");

        tab.dirty = true;
        let dirty = tab.display_label();
        assert!(dirty.ends_with("    ●"));
        assert_eq!(tab.title, "scene.glyf");
        assert_eq!(
            tab.accessibility_name(),
            "scene.glyf, unsaved, C:/work/scene.glyf"
        );
        assert_eq!(
            tab.accessibility_name_for(crate::desktop::Language::Russian),
            "scene.glyf, есть несохранённые изменения, C:/work/scene.glyf"
        );
    }

    #[test]
    fn title_and_full_path_are_kept_as_distinct_values() {
        let tab = DocumentTabFrame {
            key: id!(test_tab),
            title: "scene.glyf".to_owned(),
            full_path: "C:/projects/demo/scene.glyf".to_owned(),
            dirty: false,
            active: true,
        };

        assert_eq!(tab.display_label(), "scene.glyf");
        assert_eq!(tab.tooltip_text(), "C:/projects/demo/scene.glyf");
        assert!(
            tab.accessibility_name()
                .contains("C:/projects/demo/scene.glyf")
        );
    }
}
