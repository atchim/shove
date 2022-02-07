use log::{error, info, trace, debug, warn};
use pathdiff::diff_paths;
use regex::RegexSet;
use std::{
  borrow::Cow,
  io::ErrorKind,
  env::current_dir as cd,
  fs::{create_dir, create_dir_all},
  os::unix::fs::symlink,
  path::Path,
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
    let dest = dest.as_ref();

    {
      let src = src.path().display();
      let dest = dest.display();
      match self.unshove {
        false => info!("shoving \"{}\" into \"{}\"", src, dest),
        true => info!("unshoving \"{}\" from \"{}\"", src, dest),
      }
    }

    if self.no {
      trace!("leaving the filesystem as is");
      return;
    }

    let node = self.node(src);

    trace!("attempting to remove old dest file");
    match dest.symlink_metadata() {
      Ok(_) => {
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
            match ft.path.read_link().unwrap().is_absolute() == self.absolute {
              false => match ft.rm(self.rage) {
                Err(err) => {
                  error!("unable to remove bad dotlink: {}", err);
                  if !self.berserker {exit(1);}
                  return;
                }
                Ok(_) => debug!("removed bad dotlink"),
              }
              true => {
                debug!("dotfile already properly shoved");
                return;
              }
            }
          }
          Type::EmptyDir | Type::NonemptyDir if node && !self.unshove => {
            debug!("dotfile dir already properly shoved");
            return;
          }
          _ => match ft.rm(self.rage) {
            Err(err) => {
              error!("unable to remove dest file: {}", err);
              if !self.berserker {exit(1);}
              return;
            }
            Ok(_) => debug!("removed dest {}", ft.ty),
          }
        }
      }
      Err(err) => match err.kind() {
        ErrorKind::NotFound => trace!("dest file doesn't exist yet"),
        _ => {
          error!("unable to read dest file: {}", err);
          if !self.berserker {exit(1);}
          return;
        }
      }
    }

    if self.unshove {return;}

    trace!("attempting to create dest file");
    match node {
      false => {
        let src = src.path().canonicalize().unwrap();

        let path = match self.absolute {
          false => {
            let base = match dest.is_relative() {
              false => Cow::Borrowed(dest),
              true => Cow::Owned(cd().unwrap().join(dest)),
            };
            let base = base.parent().unwrap();
            diff_paths(&src, base).unwrap()
          }
          true => src,
        };

        match symlink(path, dest) {
          Err(err) => {
            error!("unable to create dest symlink: {}",  err);
            if !self.berserker {exit(1);}
          }
          Ok(_) => debug!("created dest symlink"),
        }
      }
      true => match create_dir(dest) {
        Err(err) => {
          error!("unable to create dest dir: {}", err);
          if !self.berserker {exit(1);}
        }
        Ok(_) => debug!("created dest dir"),
      }
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
          continue;
        }
        Ok(dot) => dot,
      };

      info!("shoving dot \"{}\"", dot.name);

      let src = dot.src;
      let dest = match dot.dest {
        Sh::Expanded {buf, s} => {
          trace!("expanded \"{}\" to \"{}\"", s, buf.display());
          Cow::Owned(buf)
        }
        Sh::Normal(dest) => Cow::Borrowed(dest),
      };

      if !dest.exists() {
        trace!("root dest dir doesn't exist yet");
        if !self.unshove {
          match create_dir_all(&dest) {
            Err(err) => {
              error!("unable to create dest root dir: {}", err);
              if !self.berserker {exit(1);}
              continue;
            }
            Ok(_) => debug!("created dest root dir"),
          }
        }
      }

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
          warn!("ignoring path \"{}\"", path.display());
        }
        !ignored
      });

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
    }
  }
}
