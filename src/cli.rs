use clap::{ArgEnum, Parser};
use std::str::FromStr;

#[derive(ArgEnum, Clone, Copy, Debug)]
pub enum ColorWhen {
  Always,
  Auto,
  Never,
}

/// Stow, but angry.
#[derive(Debug, Parser)]
pub struct Opts {
  /// Shove with absolute path.
  #[clap(long, short = '/', value_name = "SWITCH")]
  pub absolute: Option<Switch>,

  /// Do not die on error.
  #[clap(long, short, value_name = "SWITCH")]
  pub berserker: Option<Switch>,

  /// When to use colorful output.
  #[clap(arg_enum, default_value = "auto", long, short, value_name = "WHEN")]
  pub color: ColorWhen,

  /// Dots to be shoved.
  #[clap(value_name = "DOT")]
  pub dots: Vec<String>,

  /// Limit depth level to shove.
  #[clap(long, short, value_name = "LEVEL")]
  pub depth: Option<usize>,

  /// Follow links.
  #[clap(long, short, value_name = "SWITCH")]
  pub follow: Option<Switch>,

  /// Do not make any change to the filesystem.
  #[clap(long, short)]
  pub no: bool,

  /// Rage to remove files.
  #[clap(long, short, value_name = "LEVEL")]
  pub rage: Option<usize>,

  /// Unshove dots.
  #[clap(long, short)]
  pub unshove: bool,

  /// Increase verbosity.
  #[clap(parse(from_occurrences), short)]
  verbose: usize,

  /// Decrease verbosity.
  #[clap(parse(from_occurrences), short)]
  quiet: usize,
}

impl Opts {
  pub fn verbose(&self) -> usize {
    1 + self.verbose - self.quiet
  }
}

#[derive(Clone, Copy, Debug)]
pub enum Switch {
  Off,
  On,
}

impl From<bool> for Switch {
  fn from(b: bool) -> Self {
    match b {
      false => Switch::Off,
      true => Switch::On,
    }
  }
}

impl From<Switch> for bool {
  fn from(s: Switch) -> Self {
    match s {
      Switch::Off => false,
      Switch::On => true,
    }
  }
}

impl FromStr for Switch {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "0" | "off" => Ok(Switch::Off),
      "1" | "on" => Ok(Switch::On),
      _ => Err(format!("invalid switch value: {}", s)),
    }
  }
}
