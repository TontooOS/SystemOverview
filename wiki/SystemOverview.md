# SystemOverview

The About window of TontooOS. A fixed 320x580 card that shows the two
traffic lights, the device name, a laptop illustration and a two-column
spec grid. There is no `Titlebar` element: the card draws its own
close/minimize lights, so there is no title text and no maximize button,
and the light row doubles as the window drag handle.

Rendering is TontooUI on Vello/WGPU. The shell owns the window
background, the rounded body, the shadow rim and the frame, so this app
only paints content into the `Viewport`.

## Window Size

TontooUI keeps `frame::MARGIN` (24 logical px) of transparent shadow rim
around the window body, so the requested window size is 48 px larger per
axis than the visible card:

```rust
const WINDOW_WIDTH: u32 = about::CARD_W as u32 + 48; // 320 + 48 = 368
const WINDOW_HEIGHT: u32 = about::CARD_H as u32 + 48; // 580 + 48 = 628
```

`about::CARD_W` and `about::CARD_H` stay the card size (320x580), so the
layout constants keep describing the visible body, not the window.

## Layout

The card has two parts: the light row and the content stack.

| Part | Owner | Size |
|---|---|---|
| Light row | `AboutApp::lights` (`TrafficLights`) | 31 px (`TRAFFIC_SIZE` + 14 px padding) |
| Content | `AboutApp::content` (`VStack`) | rest of the body |

From top to bottom the content stack contains:

1. 12 px gap
2. Laptop illustration (`Resources/laptop.png`, drawn fallback), 240x217
3. 10 px gap
4. Device name (`sysinfo::device_name`, 22 px bold, centered)
5. 14 px gap
6. Spec grid (4 rows, label right-aligned, value left-aligned, 12 px)
7. Expanding `Spacer` (takes the leftover height)

Fixed gaps are `Spacer::new().min_size(px).factor(0.0)`; the trailing
filler keeps the default factor 1.0. The content is centered
horizontally and top-aligned, exactly like the old GTK box.

```rust
let mut stack = VStack::new()
  .spacing(0.0)
  .align(Align::Center)
  .child(gap(GAP_TOP))
  .child(Laptop::new(dark))
  .child(gap(GAP_LAPTOP))
  .child(
    themed_text(&sysinfo::device_name(), mode)
      .style(TextStyle::Title2)
      .size(TITLE_SIZE)
      .weight(TITLE_WEIGHT)
      .alignment(TextAlignment::Center)
      .width(CARD_W),
  )
  .child(gap(GAP_TITLE))
  .child(spec_grid(mode))
  .child(Spacer::new());
```

There is no More Info button: the card ends after the spec grid and the
trailing filler.

## Traffic Lights

`TrafficLights` in `src/about.rs` is a plain `View` (no `Titlebar`): two
17 px circles at the top left of the card, following the TontooUI
titlebar look. It reuses the shared constants from
`TontooUI::elements::titlebar`, so the colors and metrics stay in sync
with every other TontooUI app.

| Token | Value |
|---|---|
| `TRAFFIC_SIZE` | 17 px |
| `TRAFFIC_GAP` | 10 px |
| `TRAFFIC_LEFT` | 18 px |
| `TRAFFIC_CLOSE` | `#FF5F56` |
| `TRAFFIC_MINIMIZE` | `#FFBD2E` |
| `TRAFFIC_INACTIVE` | `#888888` (unfocused window) |
| `TRAFFIC_GLYPH_CLOSE` | `#8A1F1A` (x bars) |
| `TRAFFIC_GLYPH_MINIMIZE` | `#8A6800` (minus bar) |

```rust
pub fn new() -> Self
pub fn set_rect(&mut self, x: f32, y: f32, width: f32)
pub fn set_hover(&mut self, x: f32, y: f32)
pub fn set_focused(&mut self, focused: bool)
pub fn press(&mut self, x: f32, y: f32) -> Option<TrafficAction>
pub fn drag_rect(&self) -> (f32, f32, f32, f32)
```

- `press` returns `TrafficAction::Close` or `TrafficAction::Minimize`
  for a hit (radius + 3 px tolerance) and `None` otherwise. The app maps
  those to `WindowCommand::Close` and `WindowCommand::Minimize` through
  `App::poll_window_command`. `Maximize` is never returned: there is no
  third light.
- The hover cluster spans both lights including the gap, so hovering
  between them reveals both glyphs while clicks stay precise. Glyphs are
  68% of the button diameter, 2.2 px thick rounded bars; the x is two
  bars rotated 45 degrees. Unfocused windows stay plain gray with no
  glyphs, even on hover.
- `drag_rect` is the light row minus the cluster (18 + 2 x 17 + 10 px),
  returned from `App::drag_region` so the shell starts a window drag
  there and light clicks never drag.

## Spec Grid

Each row is an `HStack` with two `BasicText` labels: a fixed-width label
column, right-aligned, and a fixed-width value column, left-aligned and
wrapping so a long CPU name never widens the fixed card.

| Token | Value |
|---|---|
| `LABEL_W` | 88 px |
| `VALUE_W` | 150 px |
| `GRID_GAP` | 10 px |
| `ROW_GAP` | 4 px |

Rows, in order: `spec.chip.label`, `spec.memory.label`,
`spec.kernel.label`, and the OS name from the daemon as the label of the
fourth row with the OS version as its value. All four use
`TextStyle::Caption` (12 px) and `TextForeground::Primary`; there are no
secondary colors.

### Themed text

Every label is built through `themed_text`, never `BasicText::new`
directly:

```rust
fn themed_text(content: &str, mode: ThemeMode) -> BasicText {
  let mut text = BasicText::new(content);
  text.set_theme(mode);
  text
}
```

`BasicText` resolves `TextForeground::Primary` against its **own**
`dark` flag, which defaults to `true`. A label that skips `set_theme`
keeps painting the dark-mode text (`#D8D9D9`) after the card switched to
the white light body, which makes the whole card look blank.
`about::tests::primary_text_depends_on_the_mode` covers this.
`set_focused` is not called here: `VStack::set_focused` forwards the
window focus to every child on its own, so unfocused windows still
desaturate.

## Laptop Illustration

`Laptop` is an enum over the two variants, both measuring 240x217 so the
card never changes size when the asset is missing:

- `File(FileImage)`: the bundled `Resources/laptop.png`, shown 1:1
  (`ImageFit::Fit`, `radius(0.0)`). `dark` only feeds the missing-file
  placeholder color.
- `Drawn(DrawnLaptop)`: the fallback, painted with Vello primitives in the
  same shapes as the old GTK drawing.

`resource_path` looks in the dev tree (`./Resources`), next to the
executable (`<exe>/Resources`, `<exe>/../Resources` for the `.app`
bundle) and finally `/usr/share/systemoverview/Resources`.

| Token | Value |
|---|---|
| `BEZEL_W` x `BEZEL_H` | 176 x 112, radius 9, `#0B0B0D` |
| `SCREEN_INSET` | 6 px, radius 5, vertical gradient `#5AB2FF` to `#2E8FFF` |
| `NOTCH_W` x `NOTCH_H` | 38 x 6, radius 3, `#0B0B0D` |
| `HINGE_W` x `HINGE_H` | 148 x 5, radius 2.5, `#B9B9BE` |
| `DECK_W` x `DECK_H` | 204 x 9, radius 4.5, vertical gradient `#E4E4E8` to `#B9B9C0` |

The 204 x 126 artwork is centered in the placed rect, so the fallback
lines up with the PNG variant.

## Colors

Only the theme background and text pair is used. There are no secondary
text colors: the spec labels and values share
`TextForeground::Primary`.

| Token | Dark | Light |
|---|---|---|
| Body background | `#1B2022` | `#FFFFFF` |
| Text | `#D8D9D9` | `#272727` |

`ThemeWatcher` follows the settings daemon live. The palette
crossfades over 0.25 s, and `App::background` returns the blended body
color every frame, so a mode switch animates. Elements bake their colors
in at build time, so `AboutApp::sync_theme` rebuilds the content stack
only when `Theme::mode` changes; an idle window never re-lays out text.
`App::set_focused` is forwarded to `VStack::set_focused`, which reaches
every child.

All text uses the system font, which is SF Pro Display on TontooOS.

## Localization

Strings live in `lang/en_us.json` and `lang/de_de.json` (only these two)
in the Accessibility shape:

```json
{
  "lang": "en_us",
  "name": "SystemOverview",
  "translations": {
    "app.title": "SystemOverview",
    "name": "SystemOverview",
    "spec.chip.label": "Chip",
    "spec.memory.label": "Memory",
    "spec.kernel.label": "Kernel"
  }
}
```

`Resources/lang/` holds copies of both files: TBuild copies only
`Resources/` into the `.app` bundle, while the top-level `name` in the
root `lang/` feeds the localized `name` in `Info.tontoo`. Keep both
locations in sync.

`src/lang.rs` loads every candidate file through
`Accessibility::LangFile::from_file` and hands them to
`Accessibility::LangStore::init` with `en_us` as the fallback. The locale
comes from `LANGUAGE`, `LC_ALL`, `LANG` or `/etc/locale.conf`.
`$SYSTEMOVERVIEW_LANG_DIR` overrides the search path (dev runs of the
bare binary outside the project dir); otherwise `lang/`,
`./Resources/lang`, the exe-relative bundle paths and
`/usr/share/systemoverview/lang` are probed in that order.

| Key | en_us | de_de |
|---|---|---|
| `app.title` | `SystemOverview` | `SystemOverview` |
| `name` | `SystemOverview` | `Systemübersicht` |
| `spec.chip.label` | `Chip` | `Chip` |
| `spec.memory.label` | `Memory` | `Arbeitsspeicher` |
| `spec.kernel.label` | `Kernel` | `Kernel` |

### `t(key)`

```rust
pub fn t(key: &str) -> String
```

Returns the localized string for `key`. Returns the key itself when the
locale file or key is missing, so the UI never renders empty text.

## Hardware Detection

Facts come from the CoreSettings daemon (via the TontooOS SDK: `sdk` with
the `TontooUI`, `Accessibility` and `CoreSettings` features plus
`sdk::preinclude!()`), fetched once per process (`std::sync::OnceLock`).
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

## Packaging

`tontoo.proj` (`bundle_id: com.tontoo.systemoverview`) lets TBuild
assemble the `.app` bundle:

```bash
tbuild app /path/to/SystemOverview
```

### Icon

The project icon is a finished `Resources/icon.tico` (4.2 MB, one
recolorable layer), not a raster. TBuild passes a `.tico` source through
byte for byte, so the bundled `App/icon.tico` is the repo file, unchanged.

| Property | Value |
|---|---|
| Background | solid `#1B2022` (the TontooOS dark body color) |
| Layer | `info.circle` SF Symbol, 340 px, centered on the 1024 px canvas |
| Layer color | `#FFFFFF`, stored as `default_color` (recolorable) |
| Source assets | `COREICON_ASSETS_DIR`, or the system resources on TontooOS |

Because the symbol layer is recolorable, the runtime tint still works and
the Apple app-icon finish is added by `TicoIcon::render` rather than
baked into the stored layer.

### Bundle Contents

The bundle carries the release binary (`App/`), the icon
(`App/icon.tico` plus a `Resources/icon.tico` copy) and the remaining
`Resources/` (`laptop.png`, `lang/`). Both the language lookup and the
illustration lookup cover the bundle layout
(`<Name>.app/Resources/...`), dev checkouts (`lang/`, `Resources/`) and
installed files (`/usr/share/systemoverview/`).

## Tests

`cargo test` runs 18 tests without a display:

| Test | Covers |
|---|---|
| `about::traffic_lights_hit_both_lights_and_nothing_else` | Close/minimize hits, no maximize light |
| `about::cluster_hover_covers_the_gap_but_clicks_do_not` | Hover cluster vs precise clicks |
| `about::drag_rect_skips_the_light_cluster` | Drag rect geometry |
| `about::laptop_keeps_the_documented_box` | Both variants measure 240x217 |
| `about::content_fits_the_fixed_card` | Content never exceeds 320x580 |
| `about::card_children_match_the_layout_contract` | Stack child order and types |
| `about::primary_text_depends_on_the_mode` | Primary text really differs per mode |
| `about::spec_row_columns_are_fixed` | Row width stays inside the card |
| `lang::project_files_translate_known_keys` | Both lang files cover every card key |
| `sysinfo::*` | Memory snapping, chassis mapping, non-empty labels |

## Usage / Example

```bash
cargo run
LANG=de_DE.UTF-8 cargo run
```

The first command shows English strings, the second German strings.

## Cross References

- [MAIN.md](MAIN.md) -- wiki entry point
- TontooUI `elements::titlebar` constants -- traffic light metrics and
  colors reused by `TrafficLights`
- TontooUI `elements::BasicText` -- `set_theme` and the
  `TextForeground::Primary` resolution
- TontooUI `elements::FileImage` -- the bundled `laptop.png`
- TontooUI `theme::ThemeWatcher` -- live dark/light plus accent
- TontooUI `renderer::window::App` -- `background`, `drag_region`,
  `poll_window_command`
- Accessibility `LangFile` / `LangStore` -- the `lang/` file format
