use std::{
  collections::{HashMap, HashSet},
  io::{Read, Write},
  path::PathBuf,
};

use cargo_toml::{DependencyDescription, WorkspaceCargoToml, Cargo, DependencyRaw, PackageToml};
use clap::Parser;
use options::Options;

mod cargo_toml;
mod options;


fn get_changed_files_in_repo(opts: &Options) -> HashSet<PathBuf> {
  let repo = git2::Repository::discover(&opts.workspace_path)
    .unwrap_or_else(|e| panic!("Expected repo on path {} {}", e, opts.workspace_path));

  let object_prev = repo.revparse_single(&opts.since).unwrap();
  let object_head = repo.revparse_single("HEAD").unwrap();

  let prev_tree = object_prev
    .as_commit()
    .map(|commit| commit.tree().unwrap())
    .unwrap();
  let head_tree = object_head
    .as_commit()
    .map(|commit| commit.tree().unwrap())
    .unwrap();
  let diff = repo
    .diff_tree_to_tree(Some(&prev_tree), Some(&head_tree), None)
    .unwrap();

  let (mut hsn, hso): (HashSet<_>, HashSet<_>) = diff
    .deltas()
    .filter_map(|delta| {
      if let (Some(path), Some(pathn)) = (delta.new_file().path(), delta.old_file().path()) {
        Some((path.to_owned(), pathn.to_owned()))
      } else {
        None
      }
    })
    .unzip();
  hsn.extend(hso.into_iter());
  hsn
}

fn convert_dependency(
  deps: HashMap<String, DependencyRaw>,
) -> HashMap<String, DependencyDescription> {
  deps
    .into_iter()
    .map(|(name, dep_raw)| {
      let dd = match dep_raw {
        DependencyRaw::Single(version) => DependencyDescription {
          version: Some(version),
          ..DependencyDescription::default()
        },
        DependencyRaw::Map(dd) => dd,
      };
      (name, dd)
    })
    .collect()
}

fn get_total_packages(opts: &Options) -> Vec<Cargo> {
  let ws_root = PathBuf::from(opts.workspace_path.to_owned());
  let path = ws_root.join("Cargo.toml");
  let mut file = std::fs::File::open(&path).unwrap();
  let mut buffer = Vec::new();
  file.read_to_end(&mut buffer).unwrap();

  let value: WorkspaceCargoToml = toml::from_slice(&buffer).unwrap();

  value
    .workspace
    .members
    .into_iter()
    .map(|member| {
      let package_toml = ws_root.join(&member).join("Cargo.toml");
      let mut file = std::fs::File::open(&package_toml).unwrap();
      let mut buffer = Vec::new();
      file.read_to_end(&mut buffer).unwrap();
      let value: PackageToml = toml::from_slice(&buffer).unwrap();
      (value, member)
    })
    .map(|(toml, package_root)| Cargo {
      dependencies: toml
        .dependencies
        .map(convert_dependency)
        .unwrap_or_else(HashMap::new),
      dev_dependencies: toml
        .dev_dependencies
        .map(convert_dependency)
        .unwrap_or_else(HashMap::new),
      build_dependencies: toml
        .build_dependencies
        .map(convert_dependency)
        .unwrap_or_else(HashMap::new),
      package: toml.package,
      package_root,
    })
    .collect()
}

fn is_changed(
  package: &Cargo,
  file_set: &HashSet<PathBuf>,
  packages_in_workspace: &[Cargo],
) -> bool {
  file_set.iter().any(|file| {
    file
      .to_str()
      .map(|path| {
        let p = String::from(path);
        // println!("path {} == {}", path, package.package_root);
        p.contains(&package.package_root)
      })
      .unwrap_or_else(|| false)
  }) || package.dependencies.iter().any(|(name, _description)| {
    if let Some(package) = packages_in_workspace
      .iter()
      .find(|p| *name == p.package.name)
    {
      is_changed(package, file_set, packages_in_workspace)
    } else {
      false
    }
  })
}

fn update_dependency(
  deps: &mut HashMap<String, DependencyDescription>,
  file_set: &HashSet<PathBuf>,
  packages_in_workspace: &[Cargo],
  options: &Options,
) {
  for (name, dep) in deps {
    if let Some(p) = packages_in_workspace
      .iter()
      .find(|p| p.package.name == *name)
    {
      if is_changed(p, file_set, packages_in_workspace) || options.replace_all {
        dep.version = Some(options.new_version.to_owned())
      }
    }
  }
}

fn main() {
  let options = options::Options::parse();

  let mut packages_in_workspace = get_total_packages(&options);
  let reference = packages_in_workspace.clone();
  let file_set = get_changed_files_in_repo(&options);

  for p in packages_in_workspace.iter_mut() {
    if is_changed(p, &file_set, &reference) || options.replace_all {
      p.package.version = options.new_version.clone();
    }

    update_dependency(&mut p.dependencies, &file_set, &reference, &options);
  }

  let ws_root = PathBuf::from(options.workspace_path);
  for p in packages_in_workspace {
    let serialized = toml::to_string_pretty(&p).unwrap();
    let where_to_write = ws_root
      .join(p.package_root)
      .join("Cargo.toml")
      ;
    
    let mut file = std::fs::OpenOptions::new()
      .write(true)
      .create(true)
      .truncate(true)
      .open(where_to_write)
      .unwrap();

    file.write_all(serialized.as_bytes()).unwrap();
  }
}
