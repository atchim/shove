use shellexpand::LookupError;
use std::{borrow::Cow, env::VarError, error, fmt, path::{Path, PathBuf}};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
  err: LookupError<VarError>,
  s: String,
}

impl Error {
  pub fn new(s: &str, err: LookupError<VarError>) -> Self {
    Error {err, s: s.into()}
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "failed to expand \"{}\": {}", self.s, self.err)
  }
}

impl error::Error for Error {}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Sh<'a> {
  Expanded {buf: PathBuf, s: &'a str},
  Normal(&'a Path),
}

impl<'a> TryFrom<&'a str> for Sh<'a> {
  type Error = Error;

  fn try_from(s: &'a str) -> Result<Self, Self::Error> {
    match shellexpand::full(s) {
      Err(err) => Err(Error::new(s, err)),
      Ok(x) => Ok(match x {
        Cow::Borrowed(x) => Sh::Normal(Path::new(x)),
        Cow::Owned(x) => Sh::Expanded {buf: x.into(), s},
      }),
    }
  }
}
