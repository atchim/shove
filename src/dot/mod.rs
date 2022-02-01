mod dots;
mod sh;

use same_file::is_same_file;
pub use self::{dots::Dots, sh::{Error as ShErr, Sh}};
use std::{error, fmt, path::{PathBuf, Path}, io};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Dot<'a> {
  pub dest: Sh<'a>,
  pub name: &'a str,
  pub src: &'a Path,
}

impl<'a> Dot<'a> {
  pub fn new(name: &'a str, src: &'a str, dest: &'a str)
    -> Result<Self, Error>
  {
    let src = Path::new(src);
    if !src.exists() {
      return Err(Error::nonex(name, src));
    }

    let dest = match Sh::try_from(dest) {
      Err(err) => return Err(Error::sh(name, err)),
      Ok(sh) => sh,
    };

    let dest_ = match &dest {
      Sh::Expanded {buf, ..} => buf,
      Sh::Normal(p) => *p,
    };

    if dest_.exists() {
      match is_same_file(dest_, src) {
        Err(err) => return Err(Error::io(name, err)),
        Ok(false) => (),
        Ok(true) => return Err(Error::same(name, dest_)),
      }
    }

    Ok(Dot {dest, name, src})
  }
}

#[derive(Debug)]
pub enum ErrKind {
  IoErr(io::Error),
  NonexistentSrc(PathBuf),
  SameFile(PathBuf),
  ShErr(ShErr),
}

#[derive(Debug)]
pub struct Error {
  pub kind: ErrKind,
  pub name: String,
}

impl Error {
  pub fn io(name: &str, err: io::Error) -> Self {
    Error {kind: ErrKind::IoErr(err), name: name.to_string()}
  }

  pub fn nonex<P>(name: &str, p: P) -> Self where P: AsRef<Path> {
    Error {
      kind: ErrKind::NonexistentSrc(p.as_ref().to_path_buf()),
      name: name.to_string(),
    }
  }

  pub fn same<P>(name: &str, p: P) -> Self where P: AsRef<Path> {
    Error {
      kind: ErrKind::SameFile(p.as_ref().to_path_buf()),
      name: name.to_string(),
    }
  }

  pub fn sh(name: &str, err: ShErr) -> Self {
    Error {kind: ErrKind::ShErr(err), name: name.to_string()}
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s: String;
    let msg: &dyn fmt::Display = match &self.kind {
      ErrKind::IoErr(err) => err,
      ErrKind::NonexistentSrc(src) => {
        s = format!("nonexistent source file \"{}\"", src.display());
        &s
      }
      ErrKind::SameFile(p) => {
        s = format!(
          "source and destination paths refers to same file \"{}\"",
          p.display(),
        );
        &s
      }
      ErrKind::ShErr(err) => err,
    };
    write!(f, "in dot \"{}\": {}", self.name, msg)
  }
}

impl error::Error for Error {}
