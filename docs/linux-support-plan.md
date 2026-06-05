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

Status: planned.

- Detect KDE/Plasma at runtime using environment signals such as `XDG_CURRENT_DESKTOP`, `KDE_FULL_SESSION`, and `DESKTOP_SESSION`.
- Add a Linux platform capability model:
  - `FullSyncOnly` for GNOME, XFCE, Cinnamon, generic Wayland/X11, and unknown desktops.
  - `KdePlaceholders` for supported KDE environments.
- Implement KDE placeholder integration as a separate backend from Windows CFAPI.
- Keep non-KDE desktops on the Phase 1 full sync path.
- Add clear logging when KDE placeholder support is unavailable or disabled.

## Phase 3: UI And Packaging

Status: planned.

- Surface the active Linux sync mode in settings.
- Disable placeholder-only options outside KDE.
- Add Linux autostart support through freedesktop `.desktop` files.
- Add Linux packaging metadata and install hooks for KDE integration.
- Add integration tests for full sync and targeted KDE capability detection.
