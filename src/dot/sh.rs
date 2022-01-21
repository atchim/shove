use shellexpand::LookupError;
use std::{borrow::Cow, env::VarError, error, fmt, path::{Path, PathBuf}};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error<'a> {
  err: LookupError<VarError>,
  s: &'a str,
}

impl<'a> Error<'a> {
  pub fn new(s: &'a str, err: LookupError<VarError>) -> Self {
    Error {err, s}
  }
}

impl<'a> fmt::Display for Error<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "failed to expand: {}: {}", self.s, self.err)
  }
}

impl<'a> error::Error for Error<'a> {}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ShPath<'a> {
  Expanded {buf: PathBuf, s: &'a str},
  Normal(&'a Path),
}

impl<'a> ShPath<'a> {
  pub fn new(s: &'a str) -> Result<Self, Error> {
    match shellexpand::full(s) {
      Err(err) => Err(Error::new(s, err)),
      Ok(x) => Ok(match x {
        Cow::Borrowed(x) => ShPath::Normal(Path::new(x)),
        Cow::Owned(x) => ShPath::Expanded {buf: x.into(), s},
      }),
    }
  }
}
