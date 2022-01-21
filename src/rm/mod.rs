mod err;

pub use self::err::*;
use std::{fs::{remove_dir, remove_file}, io, path::Path};

pub struct Dotfile<'a>(&'a Path);

impl<'a> Rm for Dotfile<'a> {
  fn ft(&self) -> &str {
    "dotfile"
  }

  fn op(&self) -> io::Result<()> {
    remove_file(self.0)
  }

  fn path(&self) -> &Path {
    self.0
  }

  fn rage(&self) -> usize {
    0
  }
}

pub struct EmptyDir<'a>(&'a Path);

impl<'a> Rm for EmptyDir<'a> {
  fn ft(&self) -> &str {
    "empty directory"
  }

  fn op(&self) -> io::Result<()> {
    remove_dir(self.0)
  }

  fn path(&self) -> &Path {
    self.0
  }

  fn rage(&self) -> usize {
    2
  }
}

pub struct File<'a>(&'a Path);

impl<'a> Rm for File<'a> {
  fn ft(&self) -> &str {
    "file"
  }

  fn op(&self) -> io::Result<()> {
    remove_file(self.0)
  }

  fn path(&self) -> &Path {
    self.0
  }

  fn rage(&self) -> usize {
    2
  }
}

pub struct NonEmptyDir<'a>(&'a Path);

impl<'a> Rm for NonEmptyDir<'a> {
  fn ft(&self) -> &str {
    "non-empty directory"
  }

  fn op(&self) -> io::Result<()> {
    remove_dir(self.0)
  }

  fn path(&self) -> &Path {
    self.0
  }

  fn rage(&self) -> usize {
    3
  }
}

pub trait Rm {
  fn ft(&self) -> &str;
  fn op(&self) -> io::Result<()>;
  fn path(&self) -> &Path;
  fn rage(&self) -> usize;

  fn rm(&self, rage: usize) -> Result<(), Error> {
    match rage >= self.rage() {
      false => Err(RageError::new(self.ft(), rage, self.rage()).into()),
      true => match self.op() {
        Err(err) => Err(err.into()),
        Ok(_) => Ok(()),
      },
    }
  }
}

pub struct Symlink<'a>(&'a Path);

impl<'a> Rm for Symlink<'a> {
  fn ft(&self) -> &str {
    "symlink"
  }

  fn op(&self) -> io::Result<()> {
    remove_file(self.0)
  }

  fn path(&self) -> &Path {
    self.0
  }

  fn rage(&self) -> usize {
    1
  }
}
