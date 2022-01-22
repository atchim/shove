mod sh;

pub use self::sh::{ShPath, ShPathErr};
use serde::Deserialize;
use std::{
  collections::{btree_map::Iter as BTreeMapIter, BTreeMap},
  error::Error,
  fmt,
  ops::{Deref, DerefMut},
  path::{Path, PathBuf},
};

pub type DotMap = BTreeMap<String, Info>;
pub type DotMapItem<'a> = (&'a String, &'a Info);
pub type DotMapIter<'a> = BTreeMapIter<'a, String, Info>;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Dot<'a> {
  pub dest: ShPath<'a>,
  pub name: &'a str,
  pub src: &'a Path,
}

impl<'a> TryFrom<DotMapItem<'a>> for Dot<'a> {
  type Error = DotErr;

  fn try_from((name, info): DotMapItem<'a>) -> Result<Self, Self::Error> {
    let (src, dest) = match info {
      Info::Str(s) => (name, s),
      Info::Table {dest, src} => (src, dest),
    };

    let src = Path::new(src);
    if !src.exists() {
      return Err(DotErr::nonexistent_src(name, src.to_path_buf()))
    }

    let dest = match ShPath::try_from(dest.as_str()) {
      Err(err) => return Err(DotErr::shpath_err(name, err)),
      Ok(shpath) => shpath,
    };
    Ok(Dot {dest, name, src})
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DotErr {
  pub name: String,
  pub ty: DotErrType,
}

impl DotErr {
  pub fn nonexistent_src<P>(name: &str, src: P) -> Self where P: AsRef<Path> {
    DotErr {
      name: name.to_string(),
      ty: DotErrType::NonexistentSrc(src.as_ref().to_path_buf()),
    }
  }

  pub fn shpath_err(name: &str, err: ShPathErr) -> Self {
    DotErr {
      name: name.to_string(),
      ty: DotErrType::ShPathErr(err),
    }
  }
}

impl fmt::Display for DotErr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s: String;
    let msg: &dyn fmt::Display = match &self.ty {
      DotErrType::ShPathErr(err) => err,
      DotErrType::NonexistentSrc(src) => {
        s = format!("nonexistent source file: {}", src.display());
        &s
      }
    };
    write!(f, "dot: {}: {}", self.name, msg)
  }
}

impl Error for DotErr {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DotErrType {
  ShPathErr(ShPathErr),
  NonexistentSrc(PathBuf),
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(from = "DotMap")]
pub struct Dots(DotMap);

impl Dots {
  pub fn iter(&self) -> Iter {
    Iter(self.0.iter())
  }
}

impl Deref for Dots {
  type Target = DotMap;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Dots {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<DotMap> for Dots {
  fn from(t: DotMap) -> Self {
    Dots(t)
  }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum Info {
  Str(String),
  Table {dest: String, src: String},
}

#[derive(Clone, Debug)]
pub struct Iter<'a>(DotMapIter<'a>);

impl<'a> Iterator for Iter<'a> {
  type Item = Result<Dot<'a>, DotErr>;

  fn next(&mut self) -> Option<Self::Item> {
    Some(Dot::try_from(self.0.next()?))
  }
}
