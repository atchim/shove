use std::{borrow::Cow, path::Path};

use log::{debug, error, info, trace};
use regex::RegexSet;
use same_file::is_same_file;
use super::{cfg::Cfg, cli::Opts, dot::{Dots, ShPath}};
use walkdir::WalkDir;

pub struct Shover {
  absolute: bool,
  depth: usize,
  dots: Dots,
  follow: bool,
  ignore: Option<RegexSet>,
  no: bool,
  rage: usize,
  unshove: bool,
}

impl Shover {
  fn is_ignored<P>(&self, p: P) -> bool where P: AsRef<Path> {
    self.ignore.as_ref().map_or_else(
      || false,
      |ignore| ignore.is_match(&p.as_ref().to_string_lossy()),
    )
  }

  pub fn new(mut cfg: Cfg, opts: Opts) -> Self {
    let absolute = opts.absolute.map_or_else(|| cfg.absolute, |s| s.into());
    let depth = opts.depth.unwrap_or(cfg.depth);
    let follow = opts.follow.map_or_else(|| cfg.follow, |s| s.into());
    let no = opts.no;
    let rage = opts.rage.unwrap_or(cfg.rage);
    let unshove = opts.unshove;

    let dots = match opts.dots.is_empty() {
      false => {
        let mut dots = Dots::default();
        opts.dots.iter().for_each(|name| {
          let info = match cfg.dots.remove(name) {
            None => {error!("no dot named: {}", name); return},
            Some(info) => info,
          };
          dots.insert(name.to_owned(), info);
        });
        dots
      }
      true => cfg.dots,
    };

    let ignore = match RegexSet::new(&cfg.ignore) {
      Err(err) => {
        error!("invalid ignore regexes: {}", err);
        None
      }
      Ok(re) => Some(re),
    };

    Shover {absolute, depth, dots, follow, ignore, no, rage, unshove}
  }

  fn shove<D, S>(&self, src: S, dest: D, _depth: usize)
    where D: AsRef<Path>, S: AsRef<Path>
  {
    let src = src.as_ref();
    let dest = dest.as_ref();

    let src_display = src.display();
    let dest_display = dest.display();

    if self.no {
      info!("not shoving: {} -> {}", src_display, dest_display);
      return;
    }

    trace!(
      "attempting to shove: {} -> {}",
      src_display,
      dest_display,
    );
  }

  pub fn shove_dots(&self) {
    for dot in self.dots.iter() {
      info!("shoving dot: {}", dot.name);

      let src = dot.src();
      if !src.is_dir() {
        error!("source path is not a directory: {}", src.display());
        continue;
      }

      let dest = match dot.dest() {
        Err(err) => {
          error!("{}", err);
          continue;
        }
        Ok(dest) => match dest {
          ShPath::Expanded {buf, s} => {
            trace!("expanded: {} -> {}", s, buf.display());
            Cow::Owned(buf)
          }
          ShPath::Normal(dest) => Cow::Borrowed(dest),
        },
      };

      // Avoid removing dotfiles.
      match is_same_file(src, &dest) {
        Err(err) => {
          error!("failed to check if paths refers to the same file: {}", err);
          continue;
        }
        Ok(true) => panic!(
          "source and destination paths are the same: {}",
          dest.display(),
        ),
        _ => (),
      }

      let mut walker = WalkDir::new(src)
        .min_depth(1) // Skip first depth level.
        .follow_links(self.follow);

      // `depth` 0 means no depth limit.
      if self.depth > 0 {
        walker = walker.max_depth(self.depth);
      }

      if self.unshove {
        walker = walker.contents_first(true);
      }

      // Filter ignored entries.
      let walker = walker.into_iter().filter_entry(|entry| {
        let path = entry.path();
        let is_ignored = self.is_ignored(path);
        if is_ignored {
          debug!("ignoring path: {}", path.display());
        }
        !is_ignored
      });

      for entry in walker {
        let entry = match entry {
          Err(err) => {
            error!("{}", err);
            continue;
          }
          Ok(entry) => entry,
        };

        let dest = dest.join(entry.path().strip_prefix(src).unwrap());
        let src = entry.path();
        self.shove(src, &dest, entry.depth());
      }
    }
  }
}
