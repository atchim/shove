mod cfg;
mod cli;
mod dot;
mod log;
mod rm;
mod shover;

use clap::Parser;
use self::{cfg::Cfg, cli::Opts, shover::Shover};
use std::{fs::read_to_string, io::ErrorKind};

const CFG_FILE: &str = ".shove.toml";

fn main() {
  let opts = Opts::parse();

  let cfg: Cfg = {
    let s = match read_to_string(CFG_FILE) {
      Err(err) => match err.kind() {
        ErrorKind::NotFound => panic!("could not find: {}", CFG_FILE),
        _ => panic!("unable to read: {}: {}", CFG_FILE, err),
      },
      Ok(s) => s,
    };
    toml::from_str(&s).unwrap()
  };

  log::init(
    opts.verbose(),
    opts.berserker.map_or_else(|| cfg.berserker, |s| s.into()),
    opts.color,
  );

  Shover::new(cfg, opts).shove_dots();
}
