use serde::Serialize;
#[cfg(target_os = "linux")]
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct KdePlaceholderBackend {
    pub detected: bool,
    pub available: bool,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_menu_path: Option<String>,
}

impl KdePlaceholderBackend {
    pub fn probe() -> Self {
        #[cfg(target_os = "linux")]
        {
            let detected = is_kde_session();
            if !detected {
                return Self {
                    detected,
                    available: false,
                    reason: "KDE session was not detected".to_string(),
                    service_menu_path: None,
                };
            }

            let service_menu_path = service_menu_path();
            if service_menu_path.exists() {
                return Self {
                    detected,
                    available: true,
                    reason: "KDE service menu backend is installed".to_string(),
                    service_menu_path: Some(service_menu_path.display().to_string()),
                };
            }

            return Self {
                detected,
                available: false,
                reason: "KDE placeholder backend is not installed yet".to_string(),
                service_menu_path: Some(service_menu_path.display().to_string()),
            };
        }

        #[cfg(not(target_os = "linux"))]
        {
            Self {
                detected: false,
                available: false,
                reason: "KDE placeholder backend is only applicable on Linux".to_string(),
                service_menu_path: None,
            }
        }
    }

    pub fn ensure_installed() -> anyhow::Result<Self> {
        #[cfg(target_os = "linux")]
        {
            let detected = is_kde_session();
            if !detected {
                return Ok(Self {
                    detected,
                    available: false,
                    reason: "KDE session was not detected".to_string(),
                    service_menu_path: None,
                });
            }

            install_service_menu()?;
            Ok(Self::probe())
        }

        #[cfg(not(target_os = "linux"))]
        {
            Ok(Self::probe())
        }
    }
}

#[cfg(target_os = "linux")]
fn is_kde_session() -> bool {
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

    desktop.contains("kde") || desktop.contains("plasma") || desktop.contains("true")
}

#[cfg(target_os = "linux")]
fn service_menu_path() -> PathBuf {
    let data_dir = dirs::data_local_dir()
        .or_else(|| dirs::home_dir().map(|home| home.join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."));
    data_dir
        .join("kio")
        .join("servicemenus")
        .join("cloudreve-desktop.desktop")
}

#[cfg(target_os = "linux")]
fn install_service_menu() -> anyhow::Result<()> {
    let path = service_menu_path();
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("invalid KDE service menu path"))?;
    std::fs::create_dir_all(parent)?;

    let executable = std::env::current_exe()?;
    let escaped_executable = shell_escape(&executable.display().to_string());
    let content = service_menu_content(&escaped_executable);
    std::fs::write(&path, content)?;

    tracing::info!(
        target: "kde::placeholder_backend",
        path = %path.display(),
        "Installed KDE service menu backend"
    );
    Ok(())
}

#[cfg(target_os = "linux")]
fn shell_escape(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(target_os = "linux")]
fn service_menu_content(executable: &str) -> String {
    format!(
        r#"[Desktop Entry]
Type=Service
MimeType=inode/directory;application/octet-stream;application/x-zerosize;
X-KDE-ServiceTypes=KonqPopupMenu/Plugin
X-KDE-Submenu=Cloudreve
Actions=CloudreveSyncNow;CloudreveViewOnline;

[Desktop Action CloudreveSyncNow]
Name=Sync with Cloudreve
Name[zh_CN]=使用 Cloudreve 同步
Name[zh_TW]=使用 Cloudreve 同步
Icon=folder-sync
Exec={executable} --cloudreve-sync-now %F

[Desktop Action CloudreveViewOnline]
Name=View in Cloudreve
Name[zh_CN]=在 Cloudreve 中查看
Name[zh_TW]=在 Cloudreve 中檢視
Icon=cloud
Exec={executable} --cloudreve-view-online %f
"#
    )
}
