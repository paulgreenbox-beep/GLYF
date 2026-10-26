//! Pure enabled-state model for the standard Edit, File, and Window menus.
//!
//! The UI adapter supplies a [`MenuStateSnapshot`] and reads the resulting
//! [`MenuState`]. This module intentionally has no editor or UI dependencies.

/// Editor facts needed to decide whether standard menu commands can run.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MenuStateSnapshot {
    /// Number of undo entries for the active document.
    pub undo_len: usize,
    /// Number of redo entries for the active document.
    pub redo_len: usize,
    /// Whether the active document currently has a selection.
    pub has_selection: bool,
    /// Whether Select All can select an annotation or a part.
    pub has_selectable_content: bool,
    /// Whether the application clipboard currently contains a pasteable item.
    pub has_clipboard: bool,
    /// Number of open documents.
    pub document_count: usize,
    /// Whether the active document has unsaved changes.
    pub active_document_dirty: bool,
    /// Whether the active document has a saved path.
    pub active_document_has_path: bool,
    /// Number of open documents with unsaved changes.
    pub dirty_document_count: usize,
}

/// Standard menu groups supported by this model.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MenuGroup {
    Edit,
    File,
    Window,
}

/// Standard command identifiers, grouped by their owning menu.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MenuItem {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Duplicate,
    Delete,
    SelectAll,
    New,
    Open,
    Save,
    SaveAs,
    SaveAll,
    Reopen,
    Close,
    CloseOthers,
    CloseAll,
    NextDocument,
    PreviousDocument,
}

/// Computed enabled states for the standard menus.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MenuState {
    pub undo: bool,
    pub redo: bool,
    pub cut: bool,
    pub copy: bool,
    pub paste: bool,
    pub duplicate: bool,
    pub delete: bool,
    pub select_all: bool,
    pub new: bool,
    pub open: bool,
    pub save: bool,
    pub save_as: bool,
    pub save_all: bool,
    pub reopen: bool,
    pub close: bool,
    pub close_others: bool,
    pub close_all: bool,
    pub next_document: bool,
    pub previous_document: bool,
}

impl MenuState {
    /// Compute all menu command states from a single consistent snapshot.
    pub fn from_snapshot(snapshot: &MenuStateSnapshot) -> Self {
        let has_document = snapshot.document_count > 0;
        let has_multiple_documents = snapshot.document_count > 1;
        let has_selection = has_document && snapshot.has_selection;

        Self {
            undo: has_document && snapshot.undo_len > 0,
            redo: has_document && snapshot.redo_len > 0,
            cut: has_selection,
            copy: has_selection,
            paste: has_document && snapshot.has_clipboard,
            duplicate: has_selection,
            delete: has_selection,
            select_all: has_document && snapshot.has_selectable_content,
            new: true,
            open: true,
            // Save remains available for clean documents so users can invoke
            // Save explicitly; a new, untitled document can also be saved.
            save: has_document,
            save_as: has_document,
            save_all: snapshot.dirty_document_count > 0,
            reopen: has_document && snapshot.active_document_has_path,
            close: has_document,
            close_others: has_multiple_documents,
            close_all: has_document,
            next_document: has_multiple_documents,
            previous_document: has_multiple_documents,
        }
    }

    /// Return the enabled state of a command when its group is known.
    pub fn is_enabled(&self, group: MenuGroup, item: MenuItem) -> bool {
        match (group, item) {
            (MenuGroup::Edit, MenuItem::Undo) => self.undo,
            (MenuGroup::Edit, MenuItem::Redo) => self.redo,
            (MenuGroup::Edit, MenuItem::Cut) => self.cut,
            (MenuGroup::Edit, MenuItem::Copy) => self.copy,
            (MenuGroup::Edit, MenuItem::Paste) => self.paste,
            (MenuGroup::Edit, MenuItem::Duplicate) => self.duplicate,
            (MenuGroup::Edit, MenuItem::Delete) => self.delete,
            (MenuGroup::Edit, MenuItem::SelectAll) => self.select_all,
            (MenuGroup::File, MenuItem::New) => self.new,
            (MenuGroup::File, MenuItem::Open) => self.open,
            (MenuGroup::File, MenuItem::Save) => self.save,
            (MenuGroup::File, MenuItem::SaveAs) => self.save_as,
            (MenuGroup::File, MenuItem::SaveAll) => self.save_all,
            (MenuGroup::File, MenuItem::Reopen) => self.reopen,
            (MenuGroup::File, MenuItem::Close) => self.close,
            (MenuGroup::File, MenuItem::CloseOthers) => self.close_others,
            (MenuGroup::File, MenuItem::CloseAll) => self.close_all,
            (MenuGroup::Window, MenuItem::NextDocument) => self.next_document,
            (MenuGroup::Window, MenuItem::PreviousDocument) => self.previous_document,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_state_enables_only_global_file_commands() {
        let state = MenuState::from_snapshot(&MenuStateSnapshot::default());

        assert!(state.new);
        assert!(state.open);
        assert!(!state.undo);
        assert!(!state.redo);
        assert!(!state.cut && !state.copy && !state.paste);
        assert!(!state.save && !state.save_as && !state.save_all && !state.reopen);
        assert!(!state.close && !state.close_others && !state.close_all);
        assert!(!state.next_document && !state.previous_document);
    }

    #[test]
    fn populated_state_enables_commands_supported_by_the_snapshot() {
        let state = MenuState::from_snapshot(&MenuStateSnapshot {
            undo_len: 2,
            redo_len: 1,
            has_selection: true,
            has_selectable_content: true,
            has_clipboard: true,
            document_count: 3,
            active_document_dirty: true,
            active_document_has_path: true,
            dirty_document_count: 2,
        });

        for item in [
            MenuItem::Undo,
            MenuItem::Redo,
            MenuItem::Cut,
            MenuItem::Copy,
            MenuItem::Paste,
            MenuItem::Duplicate,
            MenuItem::Delete,
            MenuItem::SelectAll,
        ] {
            assert!(state.is_enabled(MenuGroup::Edit, item), "{item:?}");
        }
        for item in [
            MenuItem::Save,
            MenuItem::SaveAs,
            MenuItem::SaveAll,
            MenuItem::Reopen,
            MenuItem::Close,
            MenuItem::CloseOthers,
            MenuItem::CloseAll,
        ] {
            assert!(state.is_enabled(MenuGroup::File, item), "{item:?}");
        }
        assert!(state.is_enabled(MenuGroup::Window, MenuItem::NextDocument));
        assert!(state.is_enabled(MenuGroup::Window, MenuItem::PreviousDocument));
    }
}
