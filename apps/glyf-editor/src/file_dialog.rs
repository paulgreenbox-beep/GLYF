use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileDialogKind {
    OpenProject,
    SaveProject,
    OpenImage,
}

pub fn choose(kind: FileDialogKind) -> Option<PathBuf> {
    choose_for_language(kind, crate::desktop::Language::English)
}

/// Opens a native file dialog with translated title and filter descriptions.
pub fn choose_for_language(
    kind: FileDialogKind,
    language: crate::desktop::Language,
) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let russian = language == crate::desktop::Language::Russian;
        let (dialog_type, title, filter) = match kind {
            FileDialogKind::OpenProject => (
                "OpenFileDialog",
                if russian { "Открыть документ GLYF" } else { "Open GLYF document" },
                if russian {
                    "Документы GLYF (*.glyf)|*.glyf|Все файлы (*.*)|*.*"
                } else {
                    "GLYF Documents (*.glyf)|*.glyf|All Files (*.*)|*.*"
                },
            ),
            FileDialogKind::SaveProject => (
                "SaveFileDialog",
                if russian { "Сохранить документ GLYF" } else { "Save GLYF document" },
                if russian {
                    "Документы GLYF (*.glyf)|*.glyf|Все файлы (*.*)|*.*"
                } else {
                    "GLYF Documents (*.glyf)|*.glyf|All Files (*.*)|*.*"
                },
            ),
            FileDialogKind::OpenImage => (
                "OpenFileDialog",
                if russian { "Добавить изображение" } else { "Add image" },
                if russian {
                    "Изображения (*.png;*.jpg;*.jpeg;*.webp;*.bmp;*.ico)|*.png;*.jpg;*.jpeg;*.webp;*.bmp;*.ico|Все файлы (*.*)|*.*"
                } else {
                    "Images (*.png;*.jpg;*.jpeg;*.webp;*.bmp;*.ico)|*.png;*.jpg;*.jpeg;*.webp;*.bmp;*.ico|All Files (*.*)|*.*"
                },
            ),
        };
        let save_defaults = if matches!(kind, FileDialogKind::SaveProject) {
            "$dialog.DefaultExt = 'glyf'; $dialog.AddExtension = $true;"
        } else {
            ""
        };
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; $dialog = New-Object System.Windows.Forms.{dialog_type}; $dialog.Title = '{title}'; $dialog.Filter = '{filter}'; {save_defaults} if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {{ [Console]::Write($dialog.FileName) }}",
            dialog_type = dialog_type,
            title = title,
            filter = filter,
            save_defaults = save_defaults,
        );
        let output = std::process::Command::new("powershell.exe")
            .args([
                "-NoLogo",
                "-NoProfile",
                "-STA",
                "-WindowStyle",
                "Hidden",
                "-Command",
                &script,
            ])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        (!path.is_empty()).then_some(PathBuf::from(path))
    }

    #[cfg(not(windows))]
    {
        let _ = kind;
        let _ = language;
        None
    }
}
