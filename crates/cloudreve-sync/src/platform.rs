use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DesktopEnvironment {
    Windows,
    Macos,
    Kde,
    Gnome,
    Xfce,
    Cinnamon,
    Mate,
    Unknown,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncModeCapability {
    WindowsCloudFiles,
    KdePlaceholders,
    FullSyncOnly,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlatformCapabilities {
    pub os: String,
    pub desktop_environment: DesktopEnvironment,
    pub sync_mode: SyncModeCapability,
    pub placeholders_supported: bool,
    pub full_sync_supported: bool,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kde_placeholder_backend: Option<crate::kde::KdePlaceholderBackend>,
}

impl PlatformCapabilities {
    pub fn current() -> Self {
        let os = std::env::consts::OS.to_string();

        #[cfg(windows)]
        {
            return Self {
                os,
                desktop_environment: DesktopEnvironment::Windows,
                sync_mode: SyncModeCapability::WindowsCloudFiles,
                placeholders_supported: true,
                full_sync_supported: true,
                reason: "Windows Cloud Files API is available for placeholder sync".to_string(),
                kde_placeholder_backend: None,
            };
        }

        #[cfg(target_os = "linux")]
        {
            let desktop_environment = detect_linux_desktop_environment();
            let kde_backend = crate::kde::KdePlaceholderBackend::probe();
            let sync_mode = if kde_backend.available {
                SyncModeCapability::KdePlaceholders
            } else {
                SyncModeCapability::FullSyncOnly
            };
            let placeholders_supported = sync_mode == SyncModeCapability::KdePlaceholders;
            let reason = match sync_mode {
                SyncModeCapability::KdePlaceholders => {
                    "KDE desktop detected; KDE placeholder backend can be used".to_string()
                }
                SyncModeCapability::FullSyncOnly => {
                    if desktop_environment == DesktopEnvironment::Kde {
                        format!("{}; using full sync", kde_backend.reason)
                    } else {
                        "No supported Linux placeholder backend detected; using full sync"
                            .to_string()
                    }
                }
                SyncModeCapability::WindowsCloudFiles => unreachable!(),
            };

            return Self {
                os,
                desktop_environment,
                sync_mode,
                placeholders_supported,
                full_sync_supported: true,
                reason,
                kde_placeholder_backend: Some(kde_backend),
            };
        }

        #[cfg(target_os = "macos")]
        {
            return Self {
                os,
                desktop_environment: DesktopEnvironment::Macos,
                sync_mode: SyncModeCapability::FullSyncOnly,
                placeholders_supported: false,
                full_sync_supported: true,
                reason: "macOS placeholder backend is not implemented; using full sync".to_string(),
                kde_placeholder_backend: None,
            };
        }

        #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
        {
            Self {
                os,
                desktop_environment: DesktopEnvironment::Unknown,
                sync_mode: SyncModeCapability::FullSyncOnly,
                placeholders_supported: false,
                full_sync_supported: true,
                reason: "Unsupported desktop platform for placeholders; using full sync"
                    .to_string(),
                kde_placeholder_backend: None,
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn detect_linux_desktop_environment() -> DesktopEnvironment {
    let values = [
        std::env::var("XDG_CURRENT_DESKTOP").ok(),
        std::env::var("KDE_FULL_SESSION").ok(),
        std::env::var("DESKTOP_SESSION").ok(),
        std::env::var("GDMSESSION").ok(),
    ];

    let desktop = values
        .iter()
        .flatten()
        .map(|value| value.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(":");

    if desktop.contains("kde") || desktop.contains("plasma") || desktop.contains("true") {
        DesktopEnvironment::Kde
    } else if desktop.contains("gnome") {
        DesktopEnvironment::Gnome
    } else if desktop.contains("xfce") {
        DesktopEnvironment::Xfce
    } else if desktop.contains("cinnamon") {
        DesktopEnvironment::Cinnamon
    } else if desktop.contains("mate") {
        DesktopEnvironment::Mate
    } else {
        DesktopEnvironment::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn detects_kde_from_desktop_string() {
        let current = std::env::var("XDG_CURRENT_DESKTOP").ok();
        unsafe {
            std::env::set_var("XDG_CURRENT_DESKTOP", "KDE");
        }
        assert_eq!(detect_linux_desktop_environment(), DesktopEnvironment::Kde);
        unsafe {
            match current {
                Some(value) => std::env::set_var("XDG_CURRENT_DESKTOP", value),
                None => std::env::remove_var("XDG_CURRENT_DESKTOP"),
            }
        }
    }
}
