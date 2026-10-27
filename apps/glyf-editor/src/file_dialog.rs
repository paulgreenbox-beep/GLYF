use std::{fmt, path::PathBuf};
use crate::desktop::Language;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileDialogKind { OpenProject, SaveProject, OpenImage }

#[derive(Debug)]
pub struct FileDialogError(String);
impl fmt::Display for FileDialogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) }
}
impl std::error::Error for FileDialogError {}

#[cfg(windows)]
impl From<windows_uia::core::Error> for FileDialogError {
    fn from(error: windows_uia::core::Error) -> Self {
        Self(format!("{error} (HRESULT 0x{:08X})", error.code().0 as u32))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct DialogSpec {
    title: &'static str,
    filter_name: &'static str,
    pattern: &'static str,
    all_files: &'static str,
}

fn dialog_spec(kind: FileDialogKind, language: Language) -> DialogSpec {
    let russian = language == Language::Russian;
    let (title, filter_name, pattern) = match kind {
        FileDialogKind::OpenProject => (
            if russian { "Открыть документ GLYF" } else { "Open GLYF document" },
            if russian { "Документы GLYF (*.glyf)" } else { "GLYF Documents (*.glyf)" },
            "*.glyf",
        ),
        FileDialogKind::SaveProject => (
            if russian { "Сохранить документ GLYF" } else { "Save GLYF document" },
            if russian { "Документы GLYF (*.glyf)" } else { "GLYF Documents (*.glyf)" },
            "*.glyf",
        ),
        FileDialogKind::OpenImage => (
            if russian { "Добавить изображение" } else { "Add image" },
            if russian { "Изображения" } else { "Images" },
            "*.png;*.jpg;*.jpeg;*.webp;*.bmp;*.ico",
        ),
    };
    DialogSpec {
        title, filter_name, pattern,
        all_files: if russian { "Все файлы (*.*)" } else { "All Files (*.*)" },
    }
}

// HRESULT_FROM_WIN32(ERROR_CANCELLED), returned by IModalWindow::Show.
#[cfg(any(windows, test))]
fn is_cancelled_hresult(code: i32) -> bool { code == 0x8007_04C7_u32 as i32 }

/// Opens the system dialog. A user cancellation is not a technical failure.
pub fn choose_for_language(kind: FileDialogKind, language: Language) -> Result<Option<PathBuf>, FileDialogError> {
    #[cfg(windows)]
    { windows::choose(kind, &dialog_spec(kind, language)) }
    #[cfg(not(windows))]
    {
        let _ = (kind, language);
        Err(FileDialogError("Native file dialogs are not available on this platform".into()))
    }
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};
    use windows_uia::{
        core::{PCWSTR, PWSTR},
        Win32::{
            System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED},
            UI::Shell::{Common::COMDLG_FILTERSPEC, FileOpenDialog, FileSaveDialog, IFileDialog, FOS_FILEMUSTEXIST, FOS_FORCEFILESYSTEM, FOS_OVERWRITEPROMPT, FOS_PATHMUSTEXIST, SIGDN_FILESYSPATH},
        },
    };

    struct Apartment;
    impl Apartment {
        fn initialize() -> Result<Self, FileDialogError> {
            // Makepad initializes OLE on its Windows UI thread. A matching STA
            // initialization is balanced even when COM was already initialized.
            unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?; }
            Ok(Self)
        }
    }
    impl Drop for Apartment {
        fn drop(&mut self) { unsafe { CoUninitialize(); } }
    }

    struct ShellPath(PWSTR);
    impl Drop for ShellPath {
        fn drop(&mut self) { unsafe { CoTaskMemFree(Some(self.0.0.cast())); } }
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub(super) fn choose(kind: FileDialogKind, spec: &DialogSpec) -> Result<Option<PathBuf>, FileDialogError> {
        // Owner identity is validated by the editor's Windows platform adapter.
        let owner = crate::platform::makepad_hwnd().map_err(FileDialogError)?;
        let _apartment = Apartment::initialize()?;
        let class = if kind == FileDialogKind::SaveProject { &FileSaveDialog } else { &FileOpenDialog };
        let dialog: IFileDialog = unsafe { CoCreateInstance(class, None, CLSCTX_INPROC_SERVER)? };
        let title = wide(spec.title);
        let filter_name = wide(spec.filter_name);
        let pattern = wide(spec.pattern);
        let all_files = wide(spec.all_files);
        let all_pattern = wide("*.*");
        let default_extension = wide("glyf");
        let filters = [
            COMDLG_FILTERSPEC { pszName: PCWSTR(filter_name.as_ptr()), pszSpec: PCWSTR(pattern.as_ptr()) },
            COMDLG_FILTERSPEC { pszName: PCWSTR(all_files.as_ptr()), pszSpec: PCWSTR(all_pattern.as_ptr()) },
        ];
        unsafe {
            dialog.SetTitle(PCWSTR(title.as_ptr()))?;
            dialog.SetFileTypes(&filters)?;
            let mut options = dialog.GetOptions()? | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST;
            if kind == FileDialogKind::SaveProject {
                options |= FOS_OVERWRITEPROMPT;
                dialog.SetDefaultExtension(PCWSTR(default_extension.as_ptr()))?;
            } else {
                options |= FOS_FILEMUSTEXIST;
            }
            dialog.SetOptions(options)?;
            match dialog.Show(Some(owner)) {
                Ok(()) => {},
                Err(error) if is_cancelled_hresult(error.code().0) => return Ok(None),
                Err(error) => return Err(error.into()),
            }
            let item = dialog.GetResult()?;
            let raw = item.GetDisplayName(SIGDN_FILESYSPATH)?;
            if raw.is_null() {
                return Err(FileDialogError("The system dialog returned an empty path".into()));
            }
            let path = ShellPath(raw);
            Ok(Some(PathBuf::from(OsString::from_wide(path.0.as_wide()))))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancel_hresult_is_not_a_technical_failure() {
        assert!(is_cancelled_hresult(0x8007_04C7_u32 as i32));
        assert!(!is_cancelled_hresult(0x8007_0005_u32 as i32));
        assert!(!is_cancelled_hresult(0));
    }

    #[test]
    fn both_languages_keep_project_and_image_filters() {
        for language in [Language::English, Language::Russian] {
            assert_eq!(dialog_spec(FileDialogKind::OpenProject, language).pattern, "*.glyf");
            assert_eq!(dialog_spec(FileDialogKind::SaveProject, language).pattern, "*.glyf");
            assert_eq!(dialog_spec(FileDialogKind::OpenImage, language).pattern, "*.png;*.jpg;*.jpeg;*.webp;*.bmp;*.ico");
        }
        assert_ne!(dialog_spec(FileDialogKind::OpenProject, Language::English).title,
                   dialog_spec(FileDialogKind::OpenProject, Language::Russian).title);
    }
}