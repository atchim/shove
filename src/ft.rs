use std::{fs::{remove_dir, remove_file}, io, path::Path};
use same_file::is_same_file;
use super::rm::Rm;

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

pub fn get_rm_ft<'a>(src: &'a Path, dest: &'a Path)
-> io::Result<Box<dyn Rm + 'a>> {
  let ft = dest.metadata()?.file_type();
  Ok(if ft.is_symlink() {
    if is_same_file(dest.read_link()?, src)? {
      Box::new(Dotfile(dest))
    } else {
      Box::new(Symlink(dest))
    }
  } else if ft.is_dir() {
    if dest.read_dir()?.count() == 0 {
      Box::new(EmptyDir(dest))
    } else {
      Box::new(NonEmptyDir(dest))
    }
  } else if ft.is_file() {
    Box::new(File(dest))
  } else {
    unreachable!(); // FIXME: This is clearly reachable...
  })
}
