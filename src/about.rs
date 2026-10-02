//! About card for SystemOverview on the TontooUI renderer.
//!
//! Reproduces the macOS-style About window: the two traffic lights drawn
//! straight onto the card (no `Titlebar` element, so no title text and no
//! maximize light), the laptop illustration, the device name, the spec
//! grid and the More Info pill button. The window background, rounded
//! body and frame come from the TontooUI shell, so this module only
//! paints content.
//!
//! Colors come from the live theme palette (Dark `#1B2022` / `#D8D9D9`,
//! Light `#FFFFFF` / `#272727`); no secondary colors are invented. All
//! text uses the system font, so SF Pro Display on TontooOS.

use std::any::Any;
use std::path::PathBuf;

use crate::TontooUI::Color;
use crate::TontooUI::elements::{
  Align, BasicText, Button, ButtonShape, FileImage, GradientPaint, HStack, ImageFit, Spacer,
  TextAlignment, TextForeground, TextStyle, TrafficAction, VStack, View,
};
use crate::TontooUI::elements::titlebar::{
  TRAFFIC_CLOSE, TRAFFIC_GAP, TRAFFIC_GLYPH_CLOSE, TRAFFIC_GLYPH_MINIMIZE, TRAFFIC_INACTIVE,
  TRAFFIC_LEFT, TRAFFIC_MINIMIZE, TRAFFIC_SIZE,
};
use crate::TontooUI::kurbo::{Affine, Circle, RoundedRect};
use crate::TontooUI::peniko::{Brush, Fill};
use crate::TontooUI::renderer::{FontSystem, ImageLoader};
use crate::TontooUI::Scene;
use crate::lang;
use crate::sysinfo;

// ── geometry (logical px) ───────────────────────────────────────────

/// Visible card size. The window adds the 24 px TontooUI frame margin on
/// every side, see `main.rs`.
pub const CARD_W: f32 = 320.0;
pub const CARD_H: f32 = 580.0;
/// Traffic light row: the button plus the same 14 px padding the
/// TontooUI `Titlebar` uses.
const LIGHTS_H: f32 = TRAFFIC_SIZE + 14.0;
/// Laptop illustration box.
const LAPTOP_W: f32 = 240.0;
const LAPTOP_H: f32 = 217.0;
/// Device name size (the old UIKit title, 22 pt bold).
const TITLE_SIZE: f32 = 22.0;
const TITLE_WEIGHT: f32 = 700.0;
/// Spec grid: fixed label column, wrapping value column, 4 px row gap.
const LABEL_W: f32 = 88.0;
const VALUE_W: f32 = 150.0;
const GRID_GAP: f32 = 10.0;
const ROW_GAP: f32 = 4.0;
/// Fixed vertical gaps, matching the old card rhythm.
const GAP_TOP: f32 = 12.0;
const GAP_LAPTOP: f32 = 10.0;
const GAP_TITLE: f32 = 14.0;
const GAP_GRID: f32 = 12.0;

// ── resources ───────────────────────────────────────────────────────

/// Locate a bundled resource: dev tree, exe-relative, or installed path.
fn resource_path(name: &str) -> Option<PathBuf> {
  let mut dirs = Vec::new();
  if let Ok(cwd) = std::env::current_dir() {
    dirs.push(cwd.join("Resources"));
  }
  if let Ok(exe) = std::env::current_exe() {
    if let Some(parent) = exe.parent() {
      dirs.push(parent.join("Resources"));
      if let Some(grand) = parent.parent() {
        // .app bundle layout: <Name>.app/{App/binary, Resources/}.
        dirs.push(grand.join("Resources"));
      }
    }
  }
  dirs.push(PathBuf::from("/usr/share/systemoverview/Resources"));
  dirs
    .into_iter()
    .map(|dir| dir.join(name))
    .find(|path| path.is_file())
}

// ── traffic lights ──────────────────────────────────────────────────

/// Close and minimize traffic lights drawn on the card itself.
///
/// Same look as the TontooUI titlebar cluster: 17 px circles, 18 px
/// left margin, 10 px gap, gray while the window is unfocused and
/// filled rounded glyphs (x / minus) on a focused group hover.
pub struct TrafficLights {
  x: f32,
  y: f32,
  width: f32,
  hover: bool,
  focused: bool,
}

impl TrafficLights {
  pub fn new() -> Self {
    Self {
      x: 0.0,
      y: 0.0,
      width: 0.0,
      hover: false,
      focused: true,
    }
  }

  pub fn set_rect(&mut self, x: f32, y: f32, width: f32) {
    self.x = x;
    self.y = y;
    self.width = width;
  }

  /// Center of light `index` (0 close, 1 minimize).
  fn center(&self, index: usize) -> (f32, f32) {
    (
      self.x + TRAFFIC_LEFT
        + TRAFFIC_SIZE / 2.0
        + index as f32 * (TRAFFIC_SIZE + TRAFFIC_GAP),
      self.y + LIGHTS_H / 2.0,
    )
  }

  /// Precise click hit test (radius plus 3 px tolerance).
  fn button_at(&self, x: f32, y: f32) -> Option<TrafficAction> {
    for (index, action) in [TrafficAction::Close, TrafficAction::Minimize]
      .iter()
      .enumerate()
    {
      let (cx, cy) = self.center(index);
      let dx = x - cx;
      let dy = y - cy;
      if dx * dx + dy * dy <= (TRAFFIC_SIZE / 2.0 + 3.0).powi(2) {
        return Some(*action);
      }
    }
    None
  }

  /// Group hover covers the whole cluster including the gap, with the
  /// same 3 px tolerance, so hovering between the lights reveals both
  /// glyphs while clicks stay precise.
  fn cluster_hover(&self, x: f32, y: f32) -> bool {
    let (c0x, cy) = self.center(0);
    let (c1x, _) = self.center(1);
    let pad = 3.0;
    let x0 = c0x - TRAFFIC_SIZE / 2.0 - pad;
    let x1 = c1x + TRAFFIC_SIZE / 2.0 + pad;
    x >= x0 && x <= x1 && (y - cy).abs() <= TRAFFIC_SIZE / 2.0 + pad
  }

  pub fn set_hover(&mut self, x: f32, y: f32) {
    self.hover = self.cluster_hover(x, y);
  }

  pub fn set_focused(&mut self, focused: bool) {
    self.focused = focused;
  }

  /// Click handling. Returns the action when a light was hit.
  pub fn press(&mut self, x: f32, y: f32) -> Option<TrafficAction> {
    self.button_at(x, y)
  }

  /// Drag rect: the top row minus the light cluster, so button clicks
  /// never start a window drag.
  pub fn drag_rect(&self) -> (f32, f32, f32, f32) {
    let cut = TRAFFIC_LEFT + TRAFFIC_SIZE * 2.0 + TRAFFIC_GAP;
    (self.x + cut, self.y, (self.width - cut).max(0.0), LIGHTS_H)
  }

  /// Rounded bar centered on the origin, 68% of the button diameter long
  /// and 2.2 px thick. Rotated copies form the x glyph.
  fn glyph_bar() -> RoundedRect {
    let len = TRAFFIC_SIZE as f64 * 0.68;
    let thick = 2.2;
    RoundedRect::new(-len / 2.0, -thick / 2.0, len / 2.0, thick / 2.0, thick / 2.0)
  }

  fn draw_glyph(scene: &mut Scene, index: usize, cx: f64, cy: f64, scale: f64) {
    let (color, angles): (Color, &[f64]) = if index == 0 {
      (
        TRAFFIC_GLYPH_CLOSE,
        &[45.0_f64.to_radians(), -45.0_f64.to_radians()],
      )
    } else {
      (TRAFFIC_GLYPH_MINIMIZE, &[0.0])
    };
    for angle in angles {
      let transform = Affine::translate((cx, cy)) * Affine::rotate(*angle) * Affine::scale(scale);
      scene.fill(
        Fill::NonZero,
        transform,
        &Brush::Solid(color),
        None,
        &Self::glyph_bar(),
      );
    }
  }

  fn render(&self, scene: &mut Scene, scale: f32) {
    let px = |v: f32| v as f64 * scale as f64;
    let colors = if self.focused {
      [TRAFFIC_CLOSE, TRAFFIC_MINIMIZE]
    } else {
      [TRAFFIC_INACTIVE, TRAFFIC_INACTIVE]
    };
    for (index, color) in colors.iter().enumerate() {
      let (cx, cy) = self.center(index);
      let circle = Circle::new((px(cx), px(cy)), (TRAFFIC_SIZE / 2.0 * scale) as f64);
      scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(*color),
        None,
        &circle,
      );
      // No glyph while the window is unfocused, even on hover.
      if self.focused && self.hover {
        Self::draw_glyph(scene, index, px(cx), px(cy), scale as f64);
      }
    }
  }
}

impl Default for TrafficLights {
  fn default() -> Self {
    Self::new()
  }
}

impl View for TrafficLights {
  fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
    (self.width, LIGHTS_H)
  }

  fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, width: f32, _height: f32) {
    self.set_rect(x, y, width);
  }

  fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, _images: &mut ImageLoader<'_>) {
    self.render(scene, fonts.scale);
  }

  fn set_hover(&mut self, x: f32, y: f32) {
    self.set_hover(x, y);
  }

  fn set_focused(&mut self, focused: bool) {
    self.set_focused(focused);
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }
}

// ── laptop illustration ─────────────────────────────────────────────

/// Laptop illustration: bundled `Resources/laptop.png` when available,
/// drawn fallback otherwise. Both variants measure 240x217, so the card
/// never changes size when the asset is missing.
pub enum Laptop {
  /// Bundled PNG, shown 1:1 without scaling or rounding.
  File(FileImage),
  /// Drawn stylization: black bezel, blue display with notch, aluminum
  /// hinge and base.
  Drawn(DrawnLaptop),
}

impl Laptop {
  /// `dark` only affects the missing-file placeholder of the PNG
  /// variant; the drawn variant has no theme-dependent color.
  pub fn new(dark: bool) -> Self {
    match resource_path("laptop.png") {
      Some(path) => {
        let mut image =
          FileImage::new(path, LAPTOP_W, LAPTOP_H).fit(ImageFit::Fit).radius(0.0);
        image.set_theme(dark);
        Self::File(image)
      }
      None => Self::Drawn(DrawnLaptop::new()),
    }
  }
}

impl Default for Laptop {
  fn default() -> Self {
    Self::new(true)
  }
}

impl View for Laptop {
  fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
    match self {
      Self::File(image) => image.measure(fonts),
      Self::Drawn(drawn) => drawn.measure(fonts),
    }
  }

  fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, width: f32, height: f32) {
    match self {
      Self::File(image) => image.place(fonts, x, y, width, height),
      Self::Drawn(drawn) => drawn.place(fonts, x, y, width, height),
    }
  }

  fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, images: &mut ImageLoader<'_>) {
    match self {
      Self::File(image) => image.draw(scene, fonts, images),
      Self::Drawn(drawn) => drawn.draw(scene, fonts, images),
    }
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }
}

/// Drawn laptop, centered in the placed rect. Used only when
/// `Resources/laptop.png` is missing.
pub struct DrawnLaptop {
  rect: (f32, f32, f32, f32),
}

/// Bezel size (the GTK version drew 176x112).
const BEZEL_W: f32 = 176.0;
const BEZEL_H: f32 = 112.0;
const BEZEL_RADIUS: f32 = 9.0;
const BEZEL_COLOR: Color = Color::from_rgb8(0x0b, 0x0b, 0x0d);
/// Screen inset inside the bezel.
const SCREEN_INSET: f32 = 6.0;
const SCREEN_RADIUS: f32 = 5.0;
const SCREEN_TOP: Color = Color::from_rgb8(0x5a, 0xb2, 0xff);
const SCREEN_BOTTOM: Color = Color::from_rgb8(0x2e, 0x8f, 0xff);
/// Notch width, drawn at the top edge of the screen.
const NOTCH_W: f32 = 38.0;
const NOTCH_H: f32 = 6.0;
/// Hinge below the bezel.
const HINGE_W: f32 = 148.0;
const HINGE_H: f32 = 5.0;
const HINGE_COLOR: Color = Color::from_rgb8(0xb9, 0xb9, 0xbe);
/// Base (deck) below the hinge.
const DECK_W: f32 = 204.0;
const DECK_H: f32 = 9.0;
const DECK_TOP: Color = Color::from_rgb8(0xe4, 0xe4, 0xe8);
const DECK_BOTTOM: Color = Color::from_rgb8(0xb9, 0xb9, 0xc0);

impl DrawnLaptop {
  pub fn new() -> Self {
    Self {
      rect: (0.0, 0.0, 0.0, 0.0),
    }
  }

  /// Fill a rounded rect with a solid color.
  fn fill(scene: &mut Scene, scale: f32, x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
    let s = scale as f64;
    let shape = RoundedRect::new(
      x as f64 * s,
      y as f64 * s,
      (x + w) as f64 * s,
      (y + h) as f64 * s,
      radius as f64 * s,
    );
    scene.fill(
      Fill::NonZero,
      Affine::IDENTITY,
      &Brush::Solid(color),
      None,
      &shape,
    );
  }

  /// Fill a rounded rect with a top-to-bottom gradient.
  fn fill_gradient(
    scene: &mut Scene,
    scale: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    top: Color,
    bottom: Color,
  ) {
    let s = scale as f64;
    let shape = RoundedRect::new(
      x as f64 * s,
      y as f64 * s,
      (x + w) as f64 * s,
      (y + h) as f64 * s,
      radius as f64 * s,
    );
    let brush = GradientPaint::vertical(vec![top, bottom]).brush(x, y, w, h, scale);
    scene.fill(Fill::NonZero, Affine::IDENTITY, &brush, None, &shape);
  }
}

impl Default for DrawnLaptop {
  fn default() -> Self {
    Self::new()
  }
}

impl View for DrawnLaptop {
  fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
    (LAPTOP_W, LAPTOP_H)
  }

  fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, width: f32, height: f32) {
    self.rect = (x, y, width.max(0.0), height.max(0.0));
  }

  fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, _images: &mut ImageLoader<'_>) {
    let (bx, by, bw, bh) = self.rect;
    if bw <= 0.0 || bh <= 0.0 {
      return;
    }
    let scale = fonts.scale;
    // Center the 204 x 126 artwork in the placed box.
    let art_w = DECK_W;
    let art_h = BEZEL_H + HINGE_H + DECK_H;
    let left = bx + (bw - art_w) / 2.0;
    let top = by + (bh - art_h) / 2.0;

    let bezel_x = left + (DECK_W - BEZEL_W) / 2.0;
    Self::fill(
      scene,
      scale,
      bezel_x,
      top,
      BEZEL_W,
      BEZEL_H,
      BEZEL_RADIUS,
      BEZEL_COLOR,
    );
    Self::fill_gradient(
      scene,
      scale,
      bezel_x + SCREEN_INSET,
      top + SCREEN_INSET,
      BEZEL_W - SCREEN_INSET * 2.0,
      BEZEL_H - SCREEN_INSET * 2.0,
      SCREEN_RADIUS,
      SCREEN_TOP,
      SCREEN_BOTTOM,
    );
    // Notch at the top edge of the screen.
    Self::fill(
      scene,
      scale,
      bezel_x + (BEZEL_W - NOTCH_W) / 2.0,
      top + SCREEN_INSET,
      NOTCH_W,
      NOTCH_H,
      NOTCH_H / 2.0,
      BEZEL_COLOR,
    );
    // Hinge and base.
    let hinge_y = top + BEZEL_H;
    Self::fill(
      scene,
      scale,
      left + (DECK_W - HINGE_W) / 2.0,
      hinge_y,
      HINGE_W,
      HINGE_H,
      HINGE_H / 2.0,
      HINGE_COLOR,
    );
    Self::fill_gradient(
      scene,
      scale,
      left,
      hinge_y + HINGE_H,
      DECK_W,
      DECK_H,
      DECK_H / 2.0,
      DECK_TOP,
      DECK_BOTTOM,
    );
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }
}

// ── card ────────────────────────────────────────────────────────────

/// One spec row: label right-aligned in a fixed column, value
/// left-aligned and wrapping in the next column (12 pt, as before).
fn spec_row(label: &str, value: &str) -> HStack {
  HStack::new().spacing(GRID_GAP).align(Align::Center).child(
    BasicText::new(label)
      .style(TextStyle::Caption)
      .width(LABEL_W)
      .alignment(TextAlignment::Trailing),
  )
  .child(
    BasicText::new(value)
      .style(TextStyle::Caption)
      .width(VALUE_W)
      .alignment(TextAlignment::Leading),
  )
}

/// The four spec rows, centered as a block.
fn spec_grid() -> VStack {
  VStack::new()
    .spacing(ROW_GAP)
    .align(Align::Center)
    .child(spec_row(
      &lang::t("spec.chip.label"),
      &sysinfo::processor(),
    ))
    .child(spec_row(
      &lang::t("spec.memory.label"),
      &sysinfo::memory_label(),
    ))
    .child(spec_row(
      &lang::t("spec.kernel.label"),
      &sysinfo::kernel_version(),
    ))
    .child(spec_row(&sysinfo::os_name(), &sysinfo::os_version()))
}

/// Fixed-height gap (flex 0, so the stack gives it no leftover space).
fn gap(px: f32) -> Spacer {
  Spacer::new().min_size(px).factor(0.0)
}

/// Build the card content for one theme: illustration, device name,
/// spec grid and the More Info pill. The traffic lights are drawn by
/// the caller (they are the drag handle, so they live outside the
/// stack), and the content starts `LIGHTS_H` below the viewport top.
///
/// The content keeps the old vertical rhythm and ends in an expanding
/// spacer, so it stays top-aligned inside the fixed 320x580 body.
pub fn build_content(accent: Color, dark: bool) -> VStack {
  let mut button = Button::new(lang::t("button.more_info"))
    .shape(ButtonShape::Capsule)
    // "More Info..." is intentionally a no-op, as before.
    .on_press(|| {});
  button.set_theme(accent, dark);

  VStack::new()
    .spacing(0.0)
    .align(Align::Center)
    .child(gap(GAP_TOP))
    .child(Laptop::new(dark))
    .child(gap(GAP_LAPTOP))
    .child(
      BasicText::new(sysinfo::device_name())
        .style(TextStyle::Title2)
        .size(TITLE_SIZE)
        .weight(TITLE_WEIGHT)
        .foreground(TextForeground::Primary)
        .alignment(TextAlignment::Center)
        .width(CARD_W),
    )
    .child(gap(GAP_TITLE))
    .child(spec_grid())
    .child(gap(GAP_GRID))
    .child(button)
    // Trailing filler: pushes the content to the top of the card.
    .child(Spacer::new())
}

/// Height of the traffic light row, the space the content must leave
/// free at the top of the viewport.
pub fn lights_height() -> f32 {
  LIGHTS_H
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn traffic_lights_hit_both_lights_and_nothing_else() {
    let mut lights = TrafficLights::new();
    lights.set_rect(0.0, 0.0, 320.0);
    let (cx, cy) = lights.center(0);
    assert_eq!(lights.press(cx, cy), Some(TrafficAction::Close));
    let (mx, my) = lights.center(1);
    assert_eq!(lights.press(mx, my), Some(TrafficAction::Minimize));
    // Far right of the row is empty: no maximize light exists.
    assert_eq!(lights.press(300.0, cy), None);
    assert_eq!(lights.press(cx, cy + 40.0), None);
  }

  #[test]
  fn cluster_hover_covers_the_gap_but_clicks_do_not() {
    let mut lights = TrafficLights::new();
    lights.set_rect(0.0, 0.0, 320.0);
    let (c0x, cy) = lights.center(0);
    let (c1x, _) = lights.center(1);
    let gap_x = (c0x + c1x) / 2.0;
    lights.set_hover(gap_x, cy);
    assert!(lights.hover, "the gap belongs to the hover cluster");
    assert_eq!(lights.press(gap_x, cy), None, "the gap fires nothing");
    lights.set_hover(300.0, cy);
    assert!(!lights.hover);
  }

  #[test]
  fn drag_rect_skips_the_light_cluster() {
    let mut lights = TrafficLights::new();
    lights.set_rect(24.0, 24.0, 320.0);
    let (x, _y, width, height) = lights.drag_rect();
    let cut = TRAFFIC_LEFT + TRAFFIC_SIZE * 2.0 + TRAFFIC_GAP;
    assert_eq!(x, 24.0 + cut);
    assert_eq!(width, 320.0 - cut);
    assert_eq!(height, LIGHTS_H);
    // The close light sits left of the drag rect, so clicks still drag.
    let (cx, _) = lights.center(0);
    assert!(cx < x);
  }

  #[test]
  fn laptop_keeps_the_documented_box() {
    let mut fonts = FontSystem::new();
    for dark in [true, false] {
      let mut laptop = Laptop::new(dark);
      assert_eq!(laptop.measure(&mut fonts), (LAPTOP_W, LAPTOP_H));
    }
  }

  #[test]
  fn card_children_match_the_layout_contract() {
    let mut stack = build_content(Color::from_rgb8(0x00, 0x7a, 0xff), true);
    // gap, laptop, gap, title, gap, grid, gap, button, filler.
    assert_eq!(stack.len(), 9);
    assert!(stack.child_mut::<Laptop>(1).is_some());
    assert!(stack.child_mut::<BasicText>(3).is_some());
    assert!(stack.child_mut::<VStack>(5).is_some());
    assert!(stack.child_mut::<Button>(7).is_some());
    // The grid holds the four spec rows.
    let grid = stack.child_mut::<VStack>(5).expect("grid");
    assert_eq!(grid.len(), 4);
    assert!(grid.child_mut::<HStack>(0).is_some());
  }

  #[test]
  fn content_fits_the_fixed_card() {
    let mut fonts = FontSystem::new();
    let mut stack = build_content(Color::from_rgb8(0x00, 0x7a, 0xff), true);
    let (w, h) = stack.measure(&mut fonts);
    // The window is fixed at 320x580, so the content must never be
    // wider or taller than the body (nothing scrolls in this card).
    assert!(w <= CARD_W, "content is {w} px wide, card is {CARD_W}");
    let total = h + lights_height();
    assert!(total <= CARD_H, "content is {total} px tall, card is {CARD_H}");
  }

  #[test]
  fn spec_row_columns_are_fixed() {
    let mut row = spec_row("Memory", "16 GB");
    assert_eq!(row.len(), 2);
    let mut fonts = FontSystem::new();
    // The row measures to label + gap + value, and stays inside the card.
    let (w, _h) = row.measure(&mut fonts);
    assert!(w <= LABEL_W + GRID_GAP + VALUE_W + 1.0, "{w}");
    assert!(LABEL_W + GRID_GAP + VALUE_W <= CARD_W);
  }
}
