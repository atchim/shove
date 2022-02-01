use log::{error, info, trace, debug, warn};
use pathdiff::diff_paths;
use regex::RegexSet;
use std::{
  borrow::Cow,
  io::ErrorKind,
  env::current_dir as cd,
  fs::create_dir,
  os::unix::fs::symlink,
  path::{Path, PathBuf},
  process::exit,
};
use super::{cfg::Cfg, cli::Opts, dot::{Dots, Sh}, ft::{Ft, Type}};
use walkdir::{DirEntry, WalkDir};

pub struct Shover {
  absolute: bool,
  berserker: bool,
  depth: usize,
  dots: Dots,
  follow: bool,
  ignore: Option<RegexSet>,
  no: bool,
  rage: usize,
  unshove: bool,
}

impl Shover {
  fn ignored<P>(&self, p: P) -> bool where P: AsRef<Path> {
    self.ignore.as_ref().map_or_else(
      || false,
      |ignore| ignore.is_match(&p.as_ref().to_string_lossy()),
    )
  }

  pub fn new(mut cfg: Cfg, opts: Opts) -> Self {
    let absolute = opts.absolute.map_or_else(|| cfg.absolute, |s| s.into());
    let berserker = opts.berserker.map_or_else(|| cfg.berserker, |s| s.into());
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
            None => {error!("no dot named \"{}\"", name); return},
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
        if !berserker {exit(1);}
        None
      }
      Ok(re) => Some(re),
    };

    Shover {
      absolute,
      berserker,
      depth,
      dots,
      follow,
      ignore,
      no,
      rage,
      unshove,
    }
  }

  fn node(&self, e: &DirEntry) -> bool {
    e.path().is_dir() && (self.depth == 0 || e.depth() < self.depth)
  }

  fn shove<P>(&self, src: &DirEntry, dest: P) where P: AsRef<Path> {
    let srcd = src.path().display();
    let dest = dest.as_ref();
    let destd = dest.display();

    match self.unshove {
      false => info!("shoving \"{}\" into \"{}\"", srcd, destd),
      true => info!("unshoving \"{}\" from \"{}\"", srcd, destd),
    }

    if self.no {
      trace!("leaving the filesystem as is");
      return;
    }

    let node = self.node(src);

    // Check if `dest` exists without traversing symlinks and remove it if
    // necessary.
    match dest.symlink_metadata() {
      Ok(_) => {
        trace!("destination file already exists");

        let ft = match Ft::new(src.path(), dest) {
          Err(err) => {
            error!("{}", err);
            if !self.berserker {exit(1);}
            return;
          }
          Ok(ft) => ft,
        };

        match ft.ty {
          Type::Dotlink if !self.unshove => {
            trace!("destination file is a dotlink");
            match ft.path.read_link().unwrap().is_absolute() == self.absolute {
              false => {
                warn!("bad dotlink; removing");
                if let Err(err) = ft.rm(self.rage) {
                  error!("unable to remove bad dotlink: {}", err);
                  if !self.berserker {exit(1);}
                  return;
                }
              }
              true => {
                debug!("dotfile already properly shoved");
                trace!("skipping");
                return;
              }
            }
          }
          Type::EmptyDir | Type::NonemptyDir if node && !self.unshove => {
            debug!("dotfile already properly shoved");
            trace!("skipping");
            return;
          }
          _ => if let Err(err) = ft.rm(self.rage) {
            error!("unable to remove destination file: {}", err);
            if !self.berserker {exit(1);}
            return;
          },
        }
      }
      Err(err) => match err.kind() {
        ErrorKind::NotFound => debug!("destination file doesn't exist yet"),
        _ => {
          error!("unable to read destination file: {}", err);
          if !self.berserker {exit(1);}
          return;
        }
      },
    }

    if self.unshove {return;}

    match node {
      false => {
        let src = src.path().canonicalize().unwrap();

        let path = match self.absolute {
          false => {
            let buf: PathBuf;
            let base = match dest.is_relative() {
              false => dest,
              true => {
                buf = cd().unwrap().join(dest);
                &buf
              }
            };
            let base = base.parent().unwrap();
            diff_paths(&src, base).unwrap()
          }
          true => src,
        };

        if let Err(err) = symlink(path, dest) {
          error!("unable to symlink: {}",  err);
          if !self.berserker {exit(1);}
        }
      }
      true => if let Err(err) = create_dir(dest) {
        error!("unable to create dir: {}", err);
        if !self.berserker {exit(1);}
      },
    }
  }

  pub fn shove_dots(&self) {
    if self.no {
      warn!("not performing any change to the filesystem");
    }

    for dot in self.dots.iter() {
      let dot = match dot {
        Err(err) => {
          error!("{}", err);
          if !self.berserker {exit(1);}
          trace!("skipping to next dot");
          continue;
        }
        Ok(dot) => dot,
      };

      info!("shoving dot \"{}\"", dot.name);

      let src = dot.src;
      let dest = match dot.dest {
        Sh::Expanded {buf, s} => {
          debug!("expanded {} -> {}", s, buf.display());
          Cow::Owned(buf)
        }
        Sh::Normal(dest) => Cow::Borrowed(dest),
      };

      let mut walker = WalkDir::new(src)
        .min_depth(1)
        .follow_links(self.follow);

      if self.depth > 0 {
        walker = walker.max_depth(self.depth);
      }

      if self.unshove {
        walker = walker.contents_first(true);
      }

      // Filter ignored entries.
      let walker = walker.into_iter().filter_entry(|entry| {
        let path = entry.path();
        let ignored = self.ignored(path);
        if ignored {
          debug!("ignoring path \"{}\"", path.display());
        }
        !ignored
      });

      trace!("walking through dotfiles");
      for entry in walker {
        let entry = match entry {
          Err(err) => {
            error!("{}", err);
            if !self.berserker {exit(1);}
            continue;
          }
          Ok(entry) => entry,
        };
        let dest = dest.join(entry.path().strip_prefix(src).unwrap());
        self.shove(&entry, &dest);
      }
      trace!("shoving process terminated for dot \"{}\"", dot.name);
    }
  }
}
