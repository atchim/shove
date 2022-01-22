use shellexpand::LookupError;
use std::{
  borrow::Cow,
  env::VarError,
  error::Error,
  fmt,
  path::{Path, PathBuf},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShPathErr {
  err: LookupError<VarError>,
  s: String,
}

impl ShPathErr {
  pub fn new(s: &str, err: LookupError<VarError>) -> Self {
    ShPathErr {err, s: s.into()}
  }
}

impl fmt::Display for ShPathErr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "failed to expand: {}: {}", self.s, self.err)
  }
}

impl Error for ShPathErr {}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ShPath<'a> {
  Expanded {buf: PathBuf, s: &'a str},
  Normal(&'a Path),
}

impl<'a> TryFrom<&'a str> for ShPath<'a> {
  type Error = ShPathErr;

  fn try_from(s: &'a str) -> Result<Self, Self::Error> {
    match shellexpand::full(s) {
      Err(err) => Err(ShPathErr::new(s.into(), err)),
      Ok(x) => Ok(match x {
        Cow::Borrowed(x) => ShPath::Normal(Path::new(x)),
        Cow::Owned(x) => ShPath::Expanded {buf: x.into(), s},
      }),
    }
  }
}
