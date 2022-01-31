mod dots;
mod sh;

use same_file::is_same_file;
pub use self::{dots::Dots, sh::{Error as ShError, Sh}};
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
      return Err(
        Error::new(name, ErrorKind::NonexistentSrc(src.to_path_buf()))
      );
    }

    let dest = match Sh::try_from(dest) {
      Err(err) => return Err(Error::new(name, ErrorKind::ShError(err))),
      Ok(sh) => sh,
    };

    let dest_ = match &dest {
      Sh::Expanded {buf, ..} => buf,
      Sh::Normal(p) => *p,
    };

    if dest_.exists() {
      match is_same_file(dest_, src) {
        Err(err) => return Err(Error::new(name, ErrorKind::IoError(err))),
        Ok(true) => return Err(
          Error::new(name, ErrorKind::SameFile(dest_.to_path_buf()))
        ),
        Ok(false) => (),
      }
    }

    Ok(Dot {dest, name, src})
  }
}

#[derive(Debug)]
pub struct Error {
  pub kind: ErrorKind,
  pub name: String,
}

impl Error {
  pub fn new(name: &str, kind: ErrorKind) -> Self {
    Error {kind, name: name.to_string()}
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s: String;
    let msg: &dyn fmt::Display = match &self.kind {
      ErrorKind::IoError(err) => err,
      ErrorKind::SameFile(p) => {
        s = format!(
          "source and destination paths refers to same file {}",
          p.display(),
        );
        &s
      }
      ErrorKind::ShError(err) => err,
      ErrorKind::NonexistentSrc(src) => {
        s = format!("nonexistent source file {}", src.display());
        &s
      }
    };
    write!(f, "in dot {}: {}", self.name, msg)
  }
}

impl error::Error for Error {}

#[derive(Debug)]
pub enum ErrorKind {
  IoError(io::Error),
  NonexistentSrc(PathBuf),
  SameFile(PathBuf),
  ShError(ShError),
}
