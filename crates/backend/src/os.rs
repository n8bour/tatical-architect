//! Operating system integration: validated `https://` URL launcher and native file/folder dialogs.

use std::path::PathBuf;
use std::process::Command;

/// Validates that `url` is a safe `https://` link without whitespace, quotes, or control chars.
pub fn validate_https_url(url: &str) -> Result<(), String> {
    if !url.starts_with("https://")
        || url.len() <= "https://".len()
        || url.contains('"')
        || url.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return Err("Link inválido".into());
    }
    Ok(())
}

/// Opens a validated `https://` URL in the default system browser.
pub fn open_url(url: &str) -> Result<(), String> {
    validate_https_url(url)?;
    #[cfg(windows)]
    let r = Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn();
    #[cfg(target_os = "macos")]
    let r = Command::new("open").arg(url).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let r = Command::new("xdg-open").arg(url).spawn();
    r.map(|_| ()).map_err(|e| e.to_string())
}

/// Opens a local file with the system's default application (e.g. an `.html` in the browser).
pub fn open_path(path: &std::path::Path) -> Result<(), String> {
    if !path.is_file() {
        return Err(format!("{}: not a file", path.display()));
    }
    #[cfg(windows)]
    let r = Command::new("explorer").arg(path).spawn();
    #[cfg(target_os = "macos")]
    let r = Command::new("open").arg(path).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let r = Command::new("xdg-open").arg(path).spawn();
    r.map(|_| ()).map_err(|e| e.to_string())
}

/// Opens a native OS file picker dialog and returns the selected file path, if any.
pub fn pick_open_file(title: &str, filters: &[(&str, &[&str])]) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new().set_title(title);
    for (name, exts) in filters {
        dialog = dialog.add_filter(*name, exts);
    }
    dialog.pick_file()
}

/// Opens a native OS save-file dialog and returns the chosen path, if any.
pub fn pick_save_file(
    title: &str,
    default_name: &str,
    filters: &[(&str, &[&str])],
) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new()
        .set_title(title)
        .set_file_name(default_name);
    for (name, exts) in filters {
        dialog = dialog.add_filter(*name, exts);
    }
    dialog.save_file()
}

/// Opens a native OS folder picker dialog and returns the chosen directory path, if any.
pub fn pick_folder(title: &str) -> Option<PathBuf> {
    rfd::FileDialog::new().set_title(title).pick_folder()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_validation_allows_clean_https_and_rejects_unsafe() {
        assert!(validate_https_url("https://github.com/rhiever/fmsave").is_ok());
        assert!(validate_https_url("http://example.com").is_err());
        assert!(validate_https_url("file:///etc/passwd").is_err());
        assert!(validate_https_url("https://").is_err());
        assert!(validate_https_url("https://example.com/a b").is_err());
        assert!(validate_https_url("https://example.com/\"bad").is_err());
        assert!(validate_https_url("https://example.com/\nfoo").is_err());
    }
}
