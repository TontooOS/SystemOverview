# SystemOverview – Wiki

SystemOverview is the TontooOS About window: a 320x580 UIKit card with
close/minimize controls, a drawn laptop illustration, a spec grid and a
More Info button. It follows the live system color scheme and loads
`en_us`/`de_de` strings from `lang/`.

- Repository: https://github.com/TontooOS/TontooOS
- License: TCL v26.1
- Version: 0.1.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Development and usage rules |
| SystemOverview | [SystemOverview.md](SystemOverview.md) | About card layout, colors and localization |

## Quick Start

Run the About window from the repository root:

```bash
cargo run
```

The window follows the GNOME system theme live (Dark `#1d1d1d`, Light
`#ececec`) and picks German strings when `LANG` starts with `de`.

See [SystemOverview.md](SystemOverview.md) for details.

## Changelog

- 2026-09-08: FishPerms trust covers `/System/Applications/**` (all system
  apps, no per-app entries); ISO stages `systemoverview.app` as folder
  plus `~/Applications/SystemOverview.app` skel link.
- 2026-09-08: Fullscreen keeps the decoration bar visible (lights reveal
  on top-edge hover, minimize gray and disabled). No app code change.
- 2026-09-08: macOS fullscreen via UIKit backend (F11 toggles, ESC exits).
  No app code change.
- 2026-09-07: `tontoo.proj` packaging (`com.tontoo.systemoverview`,
  `SystemOverview.app` verified); `Resources/lang/` mirrors root `lang/`
  for the bundle; runtime lookup covers the `.app` layout.
- 2026-09-07: App icon `Resources/app-icon.png` generated with CoreIcon
  (solid `#1d1d1d`, white `info.circle` SF Symbol, no gradient).
- 2026-09-07: Lib renamed to CoreSettings (SDK feature `CoreSettings`,
  imports via `crate::CoreSettings`).
- 2026-09-07: Daemon-only facts (no old fallbacks): CPU, RAM and OS come
  only from CoreSettings (`Unknown` when unreachable); unused lang
  value keys removed.
- 2026-09-07: Frameworks via TontooOS SDK (`sdk` with `UIKit` +
  `CoreSettings` features); facts come from the CoreSettings
  daemon first with local fallbacks; OS row shows daemon identity.
- 2026-09-07: Title is always `Laptop` or `Computer` (fallback `Computer`,
  `device.name` key removed); title spacing below increased to 14px.
- 2026-09-07: Spec values wrap to a new line (fixed 320px width kept);
  device name, CPU and kernel are detected live with lang fallbacks.
- 2026-09-07: `laptop.png` regenerated (160px laptop on padded 240x217
  canvas, symmetric margins); shown 1:1 without scaling.
- 2026-09-07: Startup disk row removed; Memory shows real RAM snapped
  down to standard levels (`src/sysinfo.rs`, real value above 25 TB).
- 2026-09-07: Kernel row shows example value `Linux Kernel 6.16.9-arch1-1`.
- 2026-09-07: Removed maximize button (custom close/minimize bar via
  `TrafficLights::without_maximize`); subtitle and footer removed; spec
  rows are now Kernel/Linux and TontooOS/26.1.
- 2026-09-07: Initial About window with UIKit, `lang/en_us.json` and
  `lang/de_de.json`.
