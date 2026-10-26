//! App-owned semantic bridge consumed by the selected GUI MCP backend.
//! Transport and process/lease ownership remain outside the editor.

use super::*;
use std::cell::{Cell, RefCell};
use windows_uia::Win32::{Foundation::{HWND, RECT}, UI::WindowsAndMessaging::GetWindowRect};
use toolkit_windows_automation::{
    AccessibilityAction, AccessibilityActionHandler, AccessibilityActionRequest,
    AccessibilityActions, AccessibilityNode, AccessibilityRect, AccessibilityRole,
    AccessibilityTree, WindowsAccessibilityBridge,
};

mod actions;
mod tree;

#[cfg(windows)]
thread_local! {
    static ACCESSIBILITY_BRIDGE: RefCell<Option<WindowsAccessibilityBridge>> = const { RefCell::new(None) };
    static ACCESSIBILITY_GENERATION: Cell<u64> = const { Cell::new(0) };
}

#[cfg(windows)]
#[derive(Clone, Debug)]
pub(crate) struct AccessibilityUiAction(pub(crate) AccessibilityActionRequest);


impl App {
    #[cfg(windows)]
    pub(crate) fn refresh_accessibility(&self, cx: &Cx) {
        let Ok(hwnd) = makepad_hwnd() else { return };
        let generation = ACCESSIBILITY_GENERATION.with(|generation| {
            let next = generation.get().wrapping_add(1);
            generation.set(next);
            next
        });
        let Ok(tree) = self.accessibility_tree(cx, hwnd, generation) else {
            return;
        };
        ACCESSIBILITY_BRIDGE.with(|slot| {
            let mut slot = slot.borrow_mut();
            if let Some(bridge) = slot.as_ref() {
                let _ = bridge.update(tree);
                return;
            }
            let handler: AccessibilityActionHandler = Arc::new(|request| {
                Cx::post_action(AccessibilityUiAction(request));
                Ok(())
            });
            if let Ok(bridge) = WindowsAccessibilityBridge::attach(hwnd, tree, handler) {
                *slot = Some(bridge);
            }
        });
    }
}
