use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct WsCargoToml {
  pub members: Vec<String>,
}
#[derive(Deserialize, Debug)]
pub struct WorkspaceCargoToml {
  pub workspace: WsCargoToml,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case", untagged)]
pub enum DependencyRaw {
  Single(String),
  Map(DependencyDescription),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct PackageToml {
  pub dependencies: Option<HashMap<String, DependencyRaw>>,
  pub dev_dependencies: Option<HashMap<String, DependencyRaw>>,
  pub build_dependencies: Option<HashMap<String, DependencyRaw>>,
  pub package: PackageDescription,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct PackageDescription {
  pub version: String,
  pub name: String,
  description: Option<String>,
  edition: Option<String>,
  authors: Option<Vec<String>>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct DependencyDescription {
  pub version: Option<String>,
  pub features: Option<Vec<String>>,
  pub default_features: Option<bool>,
  pub path: Option<String>,
  pub registry: Option<String>,
  pub package: Option<String>,
  pub git: Option<String>,
  pub branch: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Cargo {
  pub dependencies: HashMap<String, DependencyDescription>,
  pub dev_dependencies: HashMap<String, DependencyDescription>,
  pub build_dependencies: HashMap<String, DependencyDescription>,
  pub package: PackageDescription,
  #[serde(skip)]
  pub package_root: String,
}
