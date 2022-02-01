use same_file::is_same_file;
use std::{
  env::current_dir as cd,
  error,
  fmt,
  io,
  path::Path,
  fs::{remove_file, remove_dir, remove_dir_all},
};
use super::util::rel_canon;

#[derive(Debug)]
pub enum Error {
  Io(io::Error),
  Rage(RageErr),
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let err: &dyn fmt::Display;
    match self {
      Error::Io(io_err) => err = io_err,
      Error::Rage(rage_err) => err = rage_err,
    };
    write!(f, "{}", err)
  }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
  fn from(err: io::Error) -> Self {
    Error::Io(err)
  }
}

impl From<RageErr> for Error {
  fn from(err: RageErr) -> Self {
    Error::Rage(err)
  }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Ft<'a> {
  pub path: &'a Path,
  pub ty: Type,
}

impl<'a> Ft<'a> {
  fn check_rage(&self, lvl: usize, min: usize) -> Result<(), RageErr> {
    match lvl >= min {
      false => Err(RageErr::new(self.ty, lvl, min)),
      true => Ok(()),
    }
  }

  pub fn new<P>(src: &P, dest: &'a Path) -> io::Result<Self>
    where P: AsRef<Path> + ?Sized
  {
    let src = src.as_ref();
    let ty = if dest.is_symlink() {
      let link = dest.read_link()?;
      match link.exists() {
        false => Type::Symlink,
        true => {
          let link = match link.is_relative() {
            false => link,
            true => {
              let par = rel_canon(cd().unwrap(), dest).unwrap();
              let par = par.parent().unwrap();
              rel_canon(par, &link).unwrap()
            }
          };
          match is_same_file(link, src)? {
            false => Type::Symlink,
            true => Type::Dotlink,
          }
        }
      }
    } else if dest.is_dir() {
      match dest.read_dir()?.count() == 0 {
        false => Type::NonemptyDir,
        true => Type::EmptyDir,
      }
    } else {
      Type::File
    };
    Ok(Ft {path: dest, ty})
  }

  pub fn rm(&self, rage: usize) -> Result<(), Error> {
    match self.ty {
      Type::Dotlink => {
        remove_file(self.path)?;
      }
      Type::EmptyDir => {
        self.check_rage(rage, 2)?;
        remove_dir(self.path)?;
      }
      Type::File => {
        self.check_rage(rage, 2)?;
        remove_file(self.path)?;
      }
      Type::NonemptyDir => {
        self.check_rage(rage, 3)?;
        remove_dir_all(self.path)?;
      }
      Type::Symlink => {
        self.check_rage(rage, 1)?;
        remove_file(self.path)?;
      }
    }
    Ok(())
  }
}

#[derive(Clone, Debug,  Eq, Ord, PartialEq, PartialOrd)]
pub struct RageErr {
  ty: Type,
  lvl: usize,
  min: usize,
}

impl RageErr {
  pub fn new(ty: Type, lvl: usize, min: usize) -> Self {
    RageErr {ty, lvl, min}
  }
}

impl fmt::Display for RageErr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "unable to remove {}, rage level {} is less than minimum {}",
      self.ty,
      self.lvl,
      self.min,
    )
  }
}

impl error::Error for RageErr {}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Type {
  Dotlink,
  EmptyDir,
  File,
  NonemptyDir,
  Symlink,
}

impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = match self {
      Type::Dotlink => "dotfile",
      Type::EmptyDir => "empty dir",
      Type::File => "file",
      Type::NonemptyDir => "non-empty dir",
      Type::Symlink => "symlink",
    };
    write!(f, "{}", s)
  }
}
