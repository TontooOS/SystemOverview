//! SystemOverview: TontooOS About window on the TontooUI renderer.
//!
//! A 320x580 card with the two traffic lights, the laptop illustration,
//! the device name, the spec grid (Chip, Memory, Kernel, TontooOS) and a
//! More Info button. Rendering is TontooUI on Vello/WGPU: the shell owns
//! the window background, rounded body and frame, so the app only paints
//! content. The theme follows the settings daemon live through
//! `ThemeWatcher` (Dark `#1B2022` / Light `#FFFFFF`).
//!
//! The window is 48 px larger than the card on every axis: TontooUI
//! keeps `frame::MARGIN` (24 px) of transparent shadow rim around the
//! body, so `CARD_W` + 48 x `CARD_H` + 48 is a 320x580 card.

mod about;
mod lang;
mod sysinfo;

sdk::preinclude!();

use crate::TontooUI::Color;
use crate::TontooUI::elements::{TrafficAction, View, VStack};
use crate::TontooUI::renderer::window::{App, Viewport, WindowCommand, run};
use crate::TontooUI::renderer::{FontSystem, ImageLoader};
use crate::TontooUI::theme::{Theme, ThemeMode, ThemeWatcher};
use crate::TontooUI::Scene;

/// Window size: the 320x580 card plus the 24 px frame margin per side.
const WINDOW_WIDTH: u32 = about::CARD_W as u32 + 48;
const WINDOW_HEIGHT: u32 = about::CARD_H as u32 + 48;

struct AboutApp {
  /// Close and minimize lights drawn on the card itself: no TontooUI
  /// `Titlebar`, so no title text and no maximize light. The row also
  /// serves as the window drag handle.
  lights: about::TrafficLights,
  /// Everything below the light row.
  content: VStack,
  watcher: ThemeWatcher,
  /// Mode and accent the current `content` was built with.
  applied: Theme,
  focused: bool,
  bg: Color,
  command: Option<WindowCommand>,
}

impl AboutApp {
  fn new() -> Self {
    let mut watcher = ThemeWatcher::new();
    // The content bakes its colors in at build time, so probe the theme
    // before the first frame.
    watcher.poll(0.0);
    let applied = watcher.theme();
    Self {
      lights: about::TrafficLights::new(),
      content: about::build_content(applied.accent.color(), applied.mode == ThemeMode::Dark),
      watcher,
      applied,
      focused: true,
      bg: TontooUI::renderer::window::BACKGROUND,
      command: None,
    }
  }

  /// Rebuild the content when the daemon switches mode or accent: every
  /// element bakes its colors in at build time. The palette itself
  /// crossfades per frame through `App::background` and needs no
  /// rebuild, so an idle window never re-lays out text.
  fn sync_theme(&mut self) {
    let theme = self.watcher.theme();
    if theme.mode == self.applied.mode && theme.accent == self.applied.accent {
      return;
    }
    self.applied = theme;
    self.content = about::build_content(theme.accent.color(), theme.mode == ThemeMode::Dark);
    self.content.set_focused(self.focused);
  }
}

impl App for AboutApp {
  fn draw(
    &mut self,
    scene: &mut Scene,
    fonts: &mut FontSystem,
    images: &mut ImageLoader<'_>,
    viewport: Viewport,
    time_secs: f64,
  ) {
    self.watcher.poll(time_secs);
    self.watcher.set_focused(self.focused, time_secs);
    self.bg = self.watcher.palette(time_secs).bg;
    self.sync_theme();

    self
      .lights
      .set_rect(viewport.x, viewport.y, viewport.width);
    self.lights.draw(scene, fonts, images);

    // The content is top-aligned and centered: the trailing expanding
    // spacer takes the leftover height, like the old GTK box.
    let top = viewport.y + about::lights_height();
    let (content_w, _content_h) = self.content.measure(fonts);
    let x = viewport.x + ((viewport.width - content_w) / 2.0).max(0.0);
    self
      .content
      .place(fonts, x, top, content_w, (viewport.height - about::lights_height()).max(0.0));
    self.content.draw(scene, fonts, images);
  }

  fn background(&self) -> Color {
    self.bg
  }

  fn drag_region(&self) -> Option<(f32, f32, f32, f32)> {
    // The traffic light row doubles as the drag handle, minus the
    // lights themselves so button clicks never drag.
    Some(self.lights.drag_rect())
  }

  fn poll_window_command(&mut self) -> Option<WindowCommand> {
    self.command.take()
  }

  fn mouse_down(&mut self, x: f64, y: f64) {
    self.command = match self.lights.press(x as f32, y as f32) {
      Some(TrafficAction::Close) => Some(WindowCommand::Close),
      Some(TrafficAction::Minimize) => Some(WindowCommand::Minimize),
      // Unreachable: the card draws no maximize light.
      Some(TrafficAction::Maximize) => Some(WindowCommand::ToggleMaximize),
      None => None,
    };
  }

  fn mouse_move(&mut self, x: f64, y: f64) {
    self.lights.set_hover(x as f32, y as f32);
  }

  fn set_focused(&mut self, focused: bool) {
    self.focused = focused;
    self.lights.set_focused(focused);
    self.content.set_focused(focused);
  }
}

fn main() {
  lang::init();
  let title = lang::t("app.title");
  if let Err(err) = run(&title, WINDOW_WIDTH, WINDOW_HEIGHT, AboutApp::new()) {
    eprintln!("error: {err}");
    std::process::exit(1);
  }
}
