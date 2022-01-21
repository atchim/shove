use serde::Deserialize;
use super::dot::Dots;

#[derive(Clone, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub struct Cfg {
  #[serde(default)]
  pub absolute: bool,

  #[serde(default)]
  pub berserker: bool,

  #[serde(default)]
  pub depth: usize,

  #[serde(default)]
  pub dots: Dots,

  #[serde(default)]
  pub follow: bool,

  #[serde(default)]
  pub ignore: Vec<String>,

  #[serde(default)]
  pub rage: usize,
}
