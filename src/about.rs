//! About card widget for SystemOverview.
//!
//! Renders the macOS-style About window: centered laptop illustration,
//! device name, spec grid, pill button and regulatory footer. All text uses
//! the SF Pro Display family (resolved from the system font paths on
//! TontooOS) and the background follows the TontooOS standard: `#1d1d1d` in
//! Dark mode, `#ececec` in Light mode.

use crate::lang;
use gtk::prelude::*;
use crate::UIKit::apply_css;
use crate::UIKit::prelude::*;
use crate::UIKit::widget::{Widget, WidgetId, next_widget_id};

const SF_PRO: &str = "SF Pro Display";

struct Palette {
  bg: &'static str,
  fg: &'static str,
  secondary: &'static str,
  pill_bg: &'static str,
}

fn palette(dark: bool) -> Palette {
  if dark {
    Palette {
      bg: "#1d1d1d",
      fg: "#F5F5F7",
      secondary: "#A1A1A6",
      pill_bg: "#3A3A3C",
    }
  } else {
    Palette {
      bg: "#ececec",
      fg: "#1E1E1E",
      secondary: "#6E6E73",
      pill_bg: "#DEDEE0",
    }
  }
}

fn markup_label(text: &str, size: u32, weight: &str, color: &str) -> gtk::Label {
  let label = gtk::Label::new(None);
  label.set_use_markup(true);
  label.set_markup(&format!(
    "<span font_desc=\"{} {} {}\" foreground=\"{}\">{}</span>",
    SF_PRO,
    weight,
    size,
    color,
    glib::markup_escape_text(text),
  ));
  label
}

fn spacer(height: i32, expand: bool) -> gtk::Box {
  let gap = gtk::Box::new(gtk::Orientation::Vertical, 0);
  gap.set_size_request(-1, height);
  gap.set_vexpand(expand);
  gap
}

/// Locate a bundled resource: dev tree, exe-relative, or installed path.
fn resource_path(name: &str) -> Option<std::path::PathBuf> {
  let mut dirs = Vec::new();
  if let Ok(cwd) = std::env::current_dir() {
    dirs.push(cwd.join("Resources"));
  }
  if let Ok(exe) = std::env::current_exe() {
    if let Some(parent) = exe.parent() {
      dirs.push(parent.join("Resources"));
      if let Some(grand) = parent.parent() {
        dirs.push(grand.join("Resources"));
      }
    }
  }
  dirs.push(std::path::PathBuf::from("/usr/share/systemoverview/Resources"));
  dirs.into_iter().map(|dir| dir.join(name)).find(|path| path.is_file())
}

/// Laptop illustration: bundled `Resources/laptop.png` when available,
/// drawn fallback otherwise.
fn build_laptop_image() -> gtk::Widget {
  if let Some(path) = resource_path("laptop.png") {
    let picture = ImageView::new(path.to_string_lossy().to_string())
      .size(240.0, 217.0)
      .to_gtk();
    picture.set_halign(gtk::Align::Center);
    let wrap = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    wrap.set_halign(gtk::Align::Center);
    wrap.append(&picture);
    return wrap.upcast();
  }
  build_laptop()
}

/// Stylized laptop illustration: black bezel, blue display with notch,
/// aluminum hinge and base. Fallback when `Resources/laptop.png` is missing.
fn build_laptop() -> gtk::Widget {
  let wrap = gtk::Box::new(gtk::Orientation::Vertical, 0);
  wrap.set_halign(gtk::Align::Center);

  let bezel = gtk::Box::new(gtk::Orientation::Vertical, 0);
  bezel.set_size_request(176, 112);
  bezel.set_halign(gtk::Align::Center);
  apply_css(
    &bezel,
    "box { background-color: #0b0b0d; border-radius: 9px; }",
  );

  let screen = gtk::Box::new(gtk::Orientation::Vertical, 0);
  screen.set_hexpand(true);
  screen.set_vexpand(true);
  screen.set_margin_top(6);
  screen.set_margin_bottom(6);
  screen.set_margin_start(6);
  screen.set_margin_end(6);
  apply_css(
    &screen,
    "box { background: linear-gradient(180deg, #5AB2FF, #2E8FFF); border-radius: 5px; }",
  );
  bezel.append(&screen);

  let overlay = gtk::Overlay::new();
  overlay.set_halign(gtk::Align::Center);
  overlay.set_child(Some(&bezel));

  let notch = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  notch.set_size_request(38, 6);
  notch.set_halign(gtk::Align::Center);
  notch.set_valign(gtk::Align::Start);
  notch.set_margin_top(6);
  apply_css(
    &notch,
    "box { background-color: #0b0b0d; border-radius: 3px; }",
  );
  overlay.add_overlay(&notch);

  let hinge = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  hinge.set_size_request(148, 5);
  hinge.set_halign(gtk::Align::Center);
  apply_css(
    &hinge,
    "box { background-color: #B9B9BE; border-radius: 2px; }",
  );

  let deck = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  deck.set_size_request(204, 9);
  deck.set_halign(gtk::Align::Center);
  apply_css(
    &deck,
    "box { background: linear-gradient(180deg, #E4E4E8, #B9B9C0); border-radius: 4px; }",
  );

  wrap.append(&overlay);
  wrap.append(&hinge);
  wrap.append(&deck);
  wrap.upcast()
}

fn spec_row(grid: &gtk::Grid, row: i32, label: &str, value: &str, pal: &Palette) {
  let name = markup_label(label, 12, "normal", pal.secondary);
  name.set_halign(gtk::Align::End);
  name.set_xalign(1.0);
  name.set_valign(gtk::Align::Start);
  grid.attach(&name, 0, row, 1, 1);

  // Values wrap onto a new line instead of growing the fixed window.
  let data = markup_label(value, 12, "normal", pal.fg);
  data.set_halign(gtk::Align::Start);
  data.set_xalign(0.0);
  data.set_valign(gtk::Align::Start);
  data.set_wrap(true);
  data.set_wrap_mode(gtk::pango::WrapMode::WordChar);
  data.set_max_width_chars(22);
  grid.attach(&data, 1, row, 1, 1);
}

/// The About card. Implements UIKit `Widget` so `App` renders it as the
/// window content below the traffic lights.
pub struct AboutCard {
  id: WidgetId,
}

impl AboutCard {
  pub fn new() -> Self {
    Self { id: next_widget_id() }
  }
}

impl Default for AboutCard {
  fn default() -> Self {
    Self::new()
  }
}

impl Widget for AboutCard {
  fn id(&self) -> WidgetId {
    self.id
  }

  fn to_gtk(&self) -> gtk::Widget {
    let dark = crate::UIKit::app::current_color_scheme()
      .unwrap_or_else(ColorScheme::detect_system)
      == ColorScheme::Dark;
    let pal = palette(dark);

    let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    outer.set_hexpand(true);
    outer.set_vexpand(true);
    outer.set_size_request(320, 580);
    apply_css(
      &outer,
      &format!("box {{ background-color: {}; border-radius: 18px; }}", pal.bg),
    );

    outer.append(&TrafficLights::new().without_maximize().to_gtk());
    outer.append(&spacer(12, false));

    outer.append(&build_laptop_image());
    outer.append(&spacer(10, false));

    let title = markup_label(&crate::sysinfo::device_name(), 22, "bold", pal.fg);
    title.set_halign(gtk::Align::Center);
    title.set_xalign(0.5);
    outer.append(&title);
    outer.append(&spacer(14, false));

    let grid = gtk::Grid::new();
    grid.set_halign(gtk::Align::Center);
    grid.set_column_spacing(10);
    grid.set_row_spacing(4);
    spec_row(
      &grid,
      0,
      &lang::t("spec.chip.label"),
      &crate::sysinfo::processor(),
      &pal,
    );
    spec_row(
      &grid,
      1,
      &lang::t("spec.memory.label"),
      &crate::sysinfo::memory_label(),
      &pal,
    );
    spec_row(
      &grid,
      2,
      &lang::t("spec.kernel.label"),
      &crate::sysinfo::kernel_version(),
      &pal,
    );
    spec_row(
      &grid,
      3,
      &crate::sysinfo::os_name(),
      &crate::sysinfo::os_version(),
      &pal,
    );
    outer.append(&grid);
    outer.append(&spacer(12, false));

    let pill_bg = crate::UIKit::style::Color::from_hex(pal.pill_bg).unwrap_or(crate::UIKit::style::Color::GRAY);
    let pill_fg = crate::UIKit::style::Color::from_hex(pal.fg).unwrap_or(crate::UIKit::style::Color::WHITE);
    let button = Button::new(lang::t("button.more_info"))
      .background(pill_bg)
      .text_color(pill_fg)
      .corner_radius(14.0)
      .padding(18.0, 4.0)
      .on_custom("more_info");
    let button_gtk = button.to_gtk();
    let button_row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    button_row.set_halign(gtk::Align::Center);
    button_row.append(&button_gtk);
    outer.append(&button_row);
    outer.append(&spacer(0, true));
    outer.upcast()
  }
}
