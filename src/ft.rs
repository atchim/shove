use same_file::is_same_file;
use std::{error, fmt, io, path::Path, fs::{remove_file, remove_dir, remove_dir_all}};

#[derive(Debug)]
pub enum Error {
  Io(io::Error),
  Rage(RageError),
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s: &'static str;
    let err: &dyn fmt::Display;
    match self {
      Error::Io(io_err) => {
        s = "io error";
        err = io_err;
      }
      Error::Rage(rage_err) => {
        s = "rage error";
        err = rage_err;
      }
    };
    write!(f, "{}: {}", s, err)
  }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
  fn from(err: io::Error) -> Self {
    Error::Io(err)
  }
}

impl From<RageError> for Error {
  fn from(err: RageError) -> Self {
    Error::Rage(err)
  }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Ft<'a> {
  pub path: &'a Path,
  pub ty: Type,
}

impl<'a> Ft<'a> {
  fn check_rage(&self, act: usize, min: usize) -> Result<(), RageError> {
    match act >= min {
      false => Err(RageError::new(self.ty, act, min)),
      true => Ok(()),
    }
  }

  pub fn new<P>(src: &P, dest: &'a Path) -> io::Result<Self>
    where P: AsRef<Path> + ?Sized
  {
    let src = src.as_ref();
    let ty = if dest.is_symlink() {
      if is_same_file(dest.read_link()?, src)? {
        Type::Dotfile
      } else {
        Type::Symlink
      }
    } else if dest.is_dir() {
      if dest.read_dir()?.count() == 0 {
        Type::EmptyDir
      } else {
        Type::NonemptyDir
      }
    } else {
      Type::File
    };
    Ok(Ft {path: dest, ty})
  }

  pub fn rm(&self, rage: usize) -> Result<(), Error> {
    match self.ty {
      Type::Dotfile => {
        // Requires rage 0, so no need to check before remove.
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
pub struct RageError {
  ty: Type,
  act: usize,
  min: usize,
}

impl RageError {
  pub fn new(ty: Type, act: usize, min: usize) -> Self {
    RageError {ty, act, min}
  }
}

impl fmt::Display for RageError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "unable to remove {}, rage level {} is less than {}",
      self.ty,
      self.min,
      self.act,
    )
  }
}

impl error::Error for RageError {}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Type {
  Dotfile,
  EmptyDir,
  File,
  NonemptyDir,
  Symlink,
}

impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = match self {
      Type::Dotfile => "dotfile",
      Type::EmptyDir => "empty dir",
      Type::File => "file",
      Type::NonemptyDir => "non-empty dir",
      Type::Symlink => "symlink",
    };
    write!(f, "{}", s)
  }
}
