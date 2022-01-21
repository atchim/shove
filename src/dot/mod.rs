mod sh;

pub use self::sh::*;
use serde::Deserialize;
use std::{
  collections::{btree_map, BTreeMap},
  ops::{Deref, DerefMut},
  path::Path,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Dot<'a> {
  dest: &'a str,
  pub name: &'a str,
  src: &'a str,
}

impl<'a> Dot<'a> {
  pub fn dest(&'a self) -> Result<ShPath<'a>, Error<'a>> {
    ShPath::new(self.dest)
  }

  pub fn new(name: &'a str, src: &'a str, dest: &'a str) -> Self {
    Dot {dest, name, src}
  }

  pub fn src(&'a self) -> &'a Path {
    Path::new(self.src)
  }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(from = "DotsT")]
pub struct Dots(DotsT);

impl Dots {
  pub fn iter(&self) -> Iter {
    Iter(self.0.iter())
  }
}

impl Deref for Dots {
  type Target = DotsT;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Dots {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<DotsT> for Dots {
  fn from(t: DotsT) -> Self {
    Dots(t)
  }
}

pub type DotsT = BTreeMap<String, Info>;

pub type DotsTIter<'a> = btree_map::Iter<'a ,String, Info>;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum Info {
  Str(String),
  Table {dest: String, src: String},
}

#[derive(Clone, Debug)]
pub struct Iter<'a>(DotsTIter<'a>);

impl<'a> Iterator for Iter<'a> {
  type Item = Dot<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    let (name, info) = self.0.next()?;
    let (src, dest) = match info {
      Info::Str(s) => (name, s),
      Info::Table {dest, src} => (src, dest),
    };
    Some(Dot::new(name, src, dest))
  }
}
