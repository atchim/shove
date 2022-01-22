use std::{error, fmt, io, path::Path};

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

#[derive(Clone, Debug, Default,  Eq, Ord, PartialEq, PartialOrd)]
pub struct RageError {
  ft: String,
  rage: usize,
  req: usize,
}

impl RageError {
  pub fn new(ft: &str, rage: usize, req: usize) -> Self {
    RageError {ft: ft.to_string(), rage, req}
  }
}

impl fmt::Display for RageError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "unable to remove {}, rage level less than {}: {}",
      self.ft,
      self.req,
      self.rage,
    )
  }
}

impl error::Error for RageError {}

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
