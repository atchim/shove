use serde::Deserialize;
use std::{
  collections::{btree_map::Iter as BTreeMapIter, BTreeMap},
  ops::{Deref, DerefMut},
};
use super::{Dot, Error as DotError};

type DotsInt = BTreeMap<String, Info>;
type DotsIntIter<'a> = BTreeMapIter<'a, String, Info>;

#[derive(Clone, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(from = "DotsInt")]
pub struct Dots(DotsInt);

impl Dots {
  pub fn iter(&self) -> Iter {
    Iter(self.0.iter())
  }
}

impl Deref for Dots {
  type Target = DotsInt;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Dots {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<DotsInt> for Dots {
  fn from(t: DotsInt) -> Self {
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
pub struct Iter<'a>(DotsIntIter<'a>);

impl<'a> Iterator for Iter<'a> {
  type Item = Result<Dot<'a>, DotError>;

  fn next(&mut self) -> Option<Self::Item> {
    let (name, info) = self.0.next()?;
    let (src, dest) = match info {
      Info::Str(s) => (name, s),
      Info::Table {dest, src} => (src, dest),
    };
    Some(Dot::new(name, src, dest))
  }
}
