# Linux Support Plan

## Phase 1: Baseline Linux Support

Status: implemented.

- Build `cloudreve-sync` and the Tauri app on Linux.
- Keep Windows CFAPI and shell extension code behind `cfg(windows)`.
- Provide non-Windows compatibility layers for shell services, notifications, app resource roots, and CFAPI-shaped file metadata.
- On Linux and other non-Windows platforms, use full sync only:
  - Create local directories for remote folders.
  - Update inventory for remote entries.
  - Queue downloads for remote files instead of creating placeholders.
  - Keep local filesystem watching and upload/download task queues active.
- Do not expose placeholder semantics on Linux yet.

## Phase 2: KDE Placeholder Support

Status: implemented.

- Detect KDE/Plasma at runtime using environment signals such as `XDG_CURRENT_DESKTOP`, `KDE_FULL_SESSION`, and `DESKTOP_SESSION`. Implemented.
- Probe KDE placeholder backend readiness separately from KDE session detection. Implemented.
- Add a Linux platform capability model. Implemented:
  - `FullSyncOnly` for GNOME, XFCE, Cinnamon, generic Wayland/X11, and unknown desktops.
  - `KdePlaceholders` only when the KDE backend is detected and available.
- Expose platform capabilities to the Tauri UI. Implemented.
- Implement KDE placeholder integration as a separate backend from Windows CFAPI. Implemented as a KDE/Dolphin service-menu backend with CLI dispatch into the existing sync engine. This is intentionally separate from Windows CFAPI; KDE does not provide an equivalent system Cloud Files API.
- Keep non-KDE desktops on the Phase 1 full sync path.
- Add clear logging when KDE placeholder support is unavailable or disabled. Implemented.

## Phase 3: UI And Packaging

Status: planned.

- Surface the active Linux sync mode in settings.
- Disable placeholder-only options outside KDE.
- Add Linux autostart support through freedesktop `.desktop` files.
- Add Linux packaging metadata and install hooks for KDE integration.
- Add integration tests for full sync and targeted KDE capability detection.
