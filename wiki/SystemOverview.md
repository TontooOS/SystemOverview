# SystemOverview

The About window of TontooOS. A fixed 320x580 UIKit card that shows the
device name, a drawn laptop illustration, a two-column spec grid and a
pill-shaped More Info button. There is no maximize button: the window
draws its own close/minimize bar and stays resizable through the UIKit
edge handles.

## Layout

From top to bottom the card contains:

1. Traffic lights (close + minimize only, doubles as drag handle)
2. Laptop illustration (`Resources/laptop.png`, drawn fallback)
3. Device name (`Laptop` or `Computer` from `sysinfo::device_name`, 22pt bold)
4. Spec grid (label right-aligned, value left-aligned, 12pt)
5. More Info pill button (no-op action `more_info`)

The content block is vertically centered between expanding spacers, so
the card keeps its balance at the fixed 320x580 size.

```rust
let mut app = App::with_delegate(lang::t("app.title"), 320, 580, AboutDelegate);
app.no_window_bar(); // custom close/minimize bar, no maximize button
app.auto_color_scheme(); // live Dark/Light follow
app.run();
```

## Window Bar

`App` would add a three-button bar by default. SystemOverview disables
it with `no_window_bar` and embeds
`TrafficLights::new().without_maximize()` as the first card row instead.
The embedded bar keeps its `gtk::WindowHandle`, so the window is still
draggable from the top area.

| Method | Description |
|---|---|
| `show_maximize(bool)` | Show or hide the green button (default `true`) |
| `without_maximize()` | Hide the green button, keep close and minimize |

## Colors

All text uses the `SF Pro Display` family, resolved from the system font
paths (`/usr/share/fonts/OTF/SF-Pro-Display-Regular.otf`, etc.).

| Token | Dark | Light |
|---|---|---|
| Background | `#1d1d1d` | `#ececec` |
| Primary text | `#F5F5F7` | `#1E1E1E` |
| Secondary text | `#A1A1A6` | `#6E6E73` |
| Pill button | `#3A3A3C` | `#DEDEE0` |

The scheme is read from `uikit::app::current_color_scheme()` with a
`ColorScheme::detect_system()` fallback, so the card matches the live
system theme on every rebuild.

## Localization

Strings live in `lang/en_us.json` and `lang/de_de.json` (only these
two). `src/lang.rs` detects German from `LANGUAGE`, `LC_ALL`, `LANG`
or `/etc/locale.conf` and falls back to `en_us`.

`Resources/lang/` holds copies of both files: TBuild copies only
`Resources/` into the `.app` bundle (root `lang/` is used just for the
localized `name` in `Info.tontoo`). Keep both locations in sync.

| Key | en_us | de_de |
|---|---|---|
| `app.title` | `SystemOverview` | `SystemOverview` |
| `spec.chip.label` | `Chip` | `Chip` |
| `spec.memory.label` | `Memory` | `Arbeitsspeicher` |
| `spec.kernel.label` | `Kernel` | `Kernel` |
| `button.more_info` | `More Info...` | `Weitere Infos...` |

Row values come only from `src/sysinfo.rs` (daemon or local
detection, `Unknown` when unavailable) and wrap onto a new line
(`WordChar`, 22 chars max) so the fixed 320px window never grows
horizontally.

## Hardware Detection

Facts come only from the CoreSettings daemon (via the TontooOS
SDK: `sdk = { path = "../../TontooLibs/SDK", features = ["UIKit",
"CoreSettings"] }` plus `sdk::preinclude!()` and
`use UIKit::prelude::*`), fetched once per process (`OnceCell`).
Without a reachable daemon (`SETTINGS_SOCKET` or
`/run/tontoo-settings.sock`) rows show `Unknown`. The daemon provides
no kernel release or chassis type, so those two stay local:

| Function | Source | Unavailable |
|---|---|---|
| `device_name` | SMBIOS `chassis_type` (portable codes map to `Laptop`), battery check, else `Computer` | `Computer` |
| `processor` | Daemon `hardware.cpu.name` | `Unknown` |
| `memory_label` | Daemon `hardware.ram.total_gb`, snapped to levels | `Unknown` |
| `kernel_version` | `/proc/sys/kernel/osrelease` as `Linux Kernel <release>` | `Unknown` |
| `os_name` / `os_version` | Daemon `os.display_name` / `os.version` | `Unknown` |

## Memory Detection

Daemon RAM (`hardware.ram.total_gb`) is snapped down to the nearest
standard level (`2GB` … `25TB`, see `LEVELS_GB`), so the UI never shows
odd values like `63 GB`. Above 25 TB the real value is shown
(`29.3 TB`).

### `t(key)`

```rust
pub fn t(key: &str) -> String
```

Returns the localized string for `key`. Returns the key itself when the
locale file or key is missing, so the UI never renders empty text.

## Usage / Example

```bash
cargo run
LANG=de_DE.UTF-8 cargo run
```

The first command shows English strings, the second German strings. The
`more_info` action is intentionally a no-op in `AboutDelegate`.

## Packaging

`tontoo.proj` (`bundle_id: com.tontoo.systemoverview`) lets TBuild
assemble the `.app` bundle:

```bash
tbuild app /path/to/SystemOverview
```

The bundle contains the release binary (`App/`), the icon and
`Resources/` (`laptop.png`, `app-icon.png`, `lang/`). The runtime
lookup covers the bundle layout (`<Name>.app/Resources/lang`,
`<Name>.app/Resources/laptop.png`), dev checkouts (`lang/`,
`Resources/`) and installed files (`/usr/share/systemoverview/`).

## Cross References

- [MAIN.md](MAIN.md) -- wiki entry point
- UIKit `TrafficLights` (`without_maximize`) -- two-button window bar
- UIKit `App::no_window_bar` -- disables the default three-button bar
