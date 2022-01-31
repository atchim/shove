use log::{error, info, trace, debug};
use pathdiff::diff_paths;
use regex::RegexSet;
use std::{
  borrow::Cow,
  io::ErrorKind,
  fs::create_dir,
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
            None => {error!("no dot named {}", name); return},
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
        error!("invalid ignore regexes {}", err);
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
      let act = if self.unshove {"unshoving"} else {"shoving"};
      let mode = if self.no {"not "} else {""};
      info!("{}{} \"{}\" into \"{}\"", mode, act, src, dest);
      if self.no {return;}
    }

    let node = self.node(src);

    // Check if `dest` exists without traversing symlinks.
    match dest.metadata() {
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
          Type::Dotfile if !self.unshove => {
            match ft.path.read_link().unwrap().is_absolute() == self.absolute {
              false => (),
              true => {
                debug!(
                  "dotfile \"{}\" already properly shoved",
                  ft.path.display(),
                );
                return;
              }
            }
          }
          Type::EmptyDir | Type::NonemptyDir if node && !self.unshove => {
            debug!("directory {} already exists", ft.path.display());
            return;
          }
          _ => if let Err(err) = ft.rm(self.rage) {
            error!("{}", err);
            return;
          },
        }
      }
      Err(err) => match err.kind() {
        ErrorKind::NotFound => (),
        _ => error!("unable to read {}: {}", dest.display(), err),
      },
    }

    if self.unshove {return;}

    match node {
      false => {
        let path = match self.absolute {
          // FIXME: This won't work if `dest` doesn't exist.
          false => diff_paths(
            src.path().canonicalize().unwrap(),
            dest.canonicalize().unwrap(),
          ).unwrap(),
          true => src.path().canonicalize().unwrap(),
        };
        if let Err(err) = symlink(path, dest) {
          error!("unable to symlink {}: {}", src.path().display(), err);
        }
      }
      true => if let Err(err) = create_dir(dest) {
        error!("unable to create dir {}: {}", dest.display(), err);
      },
    }
  }

  pub fn shove_dots(&self) {
    for dot in self.dots.iter() {
      let dot = match dot {
        Err(err) => {
          error!("{}", err);
          if !self.berserker {exit(1);}
          continue;
        }
        Ok(dot) => dot,
      };

      info!("shoving dot {}", dot.name);

      let src = dot.src;
      let dest = match dot.dest {
        Sh::Expanded {buf, s} => {
          trace!("expanded {} -> {}", s, buf.display());
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
          debug!("ignoring path {}", path.display());
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
