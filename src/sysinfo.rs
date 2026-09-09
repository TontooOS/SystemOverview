//! System facts for the About window.
//!
//! CPU, RAM and OS identity come only from the CoreSettings daemon
//! (via the SDK), fetched once per process. Without a reachable daemon
//! the UI shows `Unknown`. The daemon provides no kernel release or
//! chassis type, so those two stay local (`/proc`, `/sys`).
//!
//! Detected RAM is always rounded down to the nearest standard level,
//! so the UI never shows odd values like `63 GB`. Above 25 TB the real
//! value is shown instead.

use once_cell::sync::OnceCell;

use crate::CoreSettings::SettingsProvider;

/// Facts fetched once from the settings daemon (if reachable).
struct DaemonFacts {
  cpu_name: Option<String>,
  ram_gb: Option<f64>,
  os_name: Option<String>,
  os_version: Option<String>,
}

static DAEMON_FACTS: OnceCell<DaemonFacts> = OnceCell::new();

fn daemon_facts() -> &'static DaemonFacts {
  DAEMON_FACTS.get_or_init(|| {
    let provider = SettingsProvider::from_env();
    let hardware = provider.hardware(false).ok();
    let os = provider.os().ok();
    DaemonFacts {
      cpu_name: hardware.as_ref().and_then(|hw| hw.cpu.name.clone()),
      ram_gb: hardware.as_ref().and_then(|hw| {
        if hw.ram.total_gb > 0.0 {
          Some(hw.ram.total_gb)
        } else {
          None
        }
      }),
      os_name: os.as_ref().map(|os| os.display_name.clone()),
      os_version: os.as_ref().map(|os| os.version.clone()),
    }
  })
}

/// Shown when the daemon is unreachable and a fact has no other source.
const UNKNOWN: &str = "Unknown";

fn non_empty(value: Option<String>) -> Option<String> {
  value.filter(|s| !s.trim().is_empty())
}

/// Standard memory levels in GB (TB levels converted: 1 TB = 1024 GB).
const LEVELS_GB: &[f64] = &[
  2.0, 4.0, 8.0, 12.0, 14.0, 16.0, 18.0, 20.0, 24.0, 28.0, 30.0, 32.0,
  38.0, 40.0, 42.0, 48.0, 52.0, 58.0, 64.0, 70.0, 80.0, 90.0, 100.0,
  128.0, 130.0, 175.0, 200.0, 256.0, 300.0, 512.0, 1024.0, 2048.0,
  3072.0, 4096.0, 5120.0, 6144.0, 7168.0, 8192.0, 9216.0, 10240.0,
  15360.0, 25600.0,
];

/// Largest snapped level (25 TB in GB). Anything above shows the real value.
const MAX_LEVEL_GB: f64 = 25600.0;

fn format_level(level_gb: f64) -> String {
  if level_gb >= 1024.0 {
    format!("{} TB", (level_gb / 1024.0) as u64)
  } else {
    format!("{} GB", level_gb as u64)
  }
}

fn format_real(total_gb: f64) -> String {
  if total_gb >= 1024.0 {
    let tb = total_gb / 1024.0;
    if tb.fract() == 0.0 {
      format!("{} TB", tb as u64)
    } else {
      format!("{:.1} TB", tb)
    }
  } else if total_gb.fract() == 0.0 {
    format!("{} GB", total_gb as u64)
  } else {
    format!("{:.1} GB", total_gb)
  }
}

/// Snap a detected total (in GB) to the display string: the largest
/// standard level at or below the real value, or the real value itself
/// when above 25 TB (or below the smallest level).
pub fn snap_memory(total_gb: f64) -> String {
  if total_gb <= 0.0 {
    return format_real(total_gb);
  }
  if total_gb > MAX_LEVEL_GB {
    return format_real(total_gb);
  }
  let mut best: Option<f64> = None;
  for &level in LEVELS_GB {
    if level <= total_gb {
      best = Some(level);
    } else {
      break;
    }
  }
  match best {
    Some(level) => format_level(level),
    None => format_real(total_gb),
  }
}

/// Display string for the Memory row: daemon RAM snapped to levels,
/// or `Unknown` when the daemon is unreachable.
pub fn memory_label() -> String {
  match daemon_facts().ram_gb {
    Some(gb) if gb > 0.0 => snap_memory(gb),
    _ => UNKNOWN.to_string(),
  }
}

/// Map an SMBIOS chassis type code to Laptop or Computer.
fn chassis_label(code: u32) -> &'static str {
  match code {
    8 | 9 | 10 | 11 | 14 | 30 | 31 | 32 => "Laptop",
    _ => "Computer",
  }
}

/// Detect the device kind: `Laptop` for portable chassis, `Computer`
/// otherwise (including VMs and containers without DMI/battery info).
pub fn device_name() -> String {
  if let Ok(content) = std::fs::read_to_string("/sys/class/dmi/id/chassis_type") {
    if let Ok(code) = content.trim().parse::<u32>() {
      return chassis_label(code).to_string();
    }
  }
  if std::fs::read_dir("/sys/class/power_supply")
    .map(|entries| {
      entries
        .filter_map(|entry| entry.ok())
        .any(|entry| entry.file_name().to_string_lossy().starts_with("BAT"))
    })
    .unwrap_or(false)
  {
    return "Laptop".to_string();
  }
  "Computer".to_string()
}

/// Processor model from the CoreSettings daemon,
/// or `Unknown` when the daemon is unreachable.
pub fn processor() -> String {
  non_empty(daemon_facts().cpu_name.clone()).unwrap_or_else(|| UNKNOWN.to_string())
}

/// Real kernel release (e.g. `6.16.9-arch1-1`), or `None` when unreadable.
pub fn kernel_release() -> Option<String> {
  let release = std::fs::read_to_string("/proc/sys/kernel/osrelease").ok()?;
  let release = release.trim().to_string();
  if release.is_empty() {
    None
  } else {
    Some(release)
  }
}

/// Kernel row value: `Linux Kernel <real release>`. The daemon provides
/// no kernel fact, so this stays local; `Unknown` when unreadable.
pub fn kernel_version() -> String {
  match kernel_release() {
    Some(release) => format!("Linux Kernel {release}"),
    None => UNKNOWN.to_string(),
  }
}

/// OS display name from the CoreSettings daemon, or `Unknown`.
pub fn os_name() -> String {
  non_empty(daemon_facts().os_name.clone()).unwrap_or_else(|| UNKNOWN.to_string())
}

/// OS version from the CoreSettings daemon, or `Unknown`.
pub fn os_version() -> String {
  non_empty(daemon_facts().os_version.clone()).unwrap_or_else(|| UNKNOWN.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn snaps_down_to_nearest_level() {
    assert_eq!(snap_memory(63.0), "58 GB");
    assert_eq!(snap_memory(64.0), "64 GB");
    assert_eq!(snap_memory(16.0), "16 GB");
    assert_eq!(snap_memory(15.9), "14 GB");
    assert_eq!(snap_memory(32.5), "32 GB");
  }

  #[test]
  fn formats_tb_levels() {
    assert_eq!(snap_memory(1024.0), "1 TB");
    assert_eq!(snap_memory(2500.0), "2 TB");
    assert_eq!(snap_memory(25600.0), "25 TB");
  }

  #[test]
  fn shows_real_value_above_25tb() {
    assert_eq!(snap_memory(30000.0), "29.3 TB");
    assert_eq!(snap_memory(32768.0), "32 TB");
  }

  #[test]
  fn shows_real_value_below_smallest_level() {
    assert_eq!(snap_memory(1.0), "1 GB");
  }

  #[test]
  fn chassis_mapping() {
    assert_eq!(chassis_label(9), "Laptop");
    assert_eq!(chassis_label(10), "Laptop");
    assert_eq!(chassis_label(31), "Laptop");
    assert_eq!(chassis_label(3), "Computer");
    assert_eq!(chassis_label(23), "Computer");
    assert_eq!(chassis_label(99), "Computer");
  }

  #[test]
  fn kernel_release_parses_when_present() {
    if std::path::Path::new("/proc/sys/kernel/osrelease").exists() {
      let label = kernel_version();
      assert!(label.starts_with("Linux Kernel "));
    }
  }

  #[test]
  fn provider_layer_returns_display_strings() {
    // With or without a daemon, every label must be non-empty.
    assert!(!processor().is_empty());
    assert!(!memory_label().is_empty());
    assert!(!os_name().is_empty());
    assert!(!os_version().is_empty());
    assert!(!kernel_version().is_empty());
    assert!(!device_name().is_empty());
  }
}
