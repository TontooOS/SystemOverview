//! SystemOverview: TontooOS About window built with UIKit.
//!
//! Shows a 320x580 card with device overview, following the live system
//! color scheme (Dark `#1d1d1d`, Light `#ececec`).

mod about;
mod lang;
mod sysinfo;

sdk::preinclude!();

use UIKit::prelude::*;

struct AboutDelegate;

impl AppDelegate for AboutDelegate {
  fn view(&self) -> Box<dyn Widget> {
    Box::new(about::AboutCard::new())
  }

  fn handle_custom(&mut self, _action: &str) {
    // "More Info..." is intentionally a no-op for now.
  }
}

fn main() {
  lang::init();
  let mut app = App::with_delegate(lang::t("app.title"), 320, 580, AboutDelegate);
  // SystemOverview draws its own close/minimize bar (no maximize button).
  app.no_window_bar();
  // Fixed 320x580 card: no resizing, no scrolling (content always fits).
  app.fixed_size();
  app.no_scroll();
  app.auto_color_scheme();
  app.run();
}
