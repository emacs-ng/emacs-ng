use std::collections::HashSet;
use std::path::PathBuf;

use super::error::BuildError;
use cargo_metadata::CargoOpt;
use cargo_metadata::Metadata;
use cargo_metadata::Node;
pub use cargo_metadata::Package;
use cargo_metadata::PackageId;

fn resolved_node(id: PackageId, metadata: &Metadata) -> Option<&Node> {
    metadata
        .resolve
        .as_ref()
        .and_then(|r| r.nodes.iter().find(|n| id == n.id))
}

// find package ids of local dependcies of package id
fn resolve_deps(id: PackageId, metadata: &Metadata) -> Vec<PackageId> {
    resolved_node(id, metadata)
        .map(|n| {
            n.dependencies
                .clone()
                .into_iter()
                .filter(|d| d.repr.starts_with("path+file:///"))
                .collect::<Vec<PackageId>>()
        })
        .map_or(Vec::new(), |ids| ids)
}

fn resolve_deps_recursively(id: PackageId, deps: &mut Vec<PackageId>, metadata: &Metadata) {
    let mut resolved_deps = resolve_deps(id, metadata);
    for id in &resolved_deps {
        resolve_deps_recursively(id.clone(), deps, metadata);
    }
    deps.append(&mut resolved_deps);
}

fn contains_crate(c: Option<&Package>, crates: &Vec<PackageId>) -> bool {
    crates
        .into_iter()
        .find(|d| c.map_or(false, |r| r.id == **d))
        .is_some()
}

fn contains_lisp_macro_crate(crates: &Vec<PackageId>, metadata: &Metadata) -> bool {
    let c = lisp_macro_crate(metadata);
    contains_crate(c, crates)
}

fn contains_emacs_sys_crate(crates: &Vec<PackageId>, metadata: &Metadata) -> bool {
    let c = emacs_sys_crate(metadata);
    contains_crate(c, crates)
}

// crate are direct dependency of root crate
// for lisp-macro in it's dependency graph should be added to
// c_export.rs
fn needed_for_c_export(id: PackageId, metadata: &Metadata) -> bool {
    let mut all_deps: Vec<PackageId> = Vec::new();
    resolve_deps_recursively(id, &mut all_deps, &metadata);
    contains_lisp_macro_crate(&all_deps, metadata)
}

// this predicte is use to check whether the root crate has lisp macro as
// its direct dependency
fn needed_for_lisp_fn_export(id: PackageId, metadata: &Metadata) -> bool {
    let deps = resolve_deps(id, &metadata);
    contains_lisp_macro_crate(&deps, metadata)
}

// needed for globals.h/DOC
fn needed_for_globals(id: PackageId, metadata: &Metadata) -> bool {
    let deps = resolve_deps(id.clone(), &metadata);
    if lisp_macro_crate(metadata).map_or(false, |c| c.id == id) {
        return false;
    }

    contains_lisp_macro_crate(&deps, metadata) | contains_emacs_sys_crate(&deps, metadata)
}

pub fn with_metadata<F: FnMut(Metadata) -> Result<(), BuildError>>(
    mut f: F,
) -> Result<(), BuildError> {
    let features = enabled_features()?;
    let metadata = cargo_metadata::MetadataCommand::new()
        .features(CargoOpt::SomeFeatures(features.to_vec()))
        .exec()?;
    f(metadata)
}

fn root_crate(metadata: &Metadata) -> Result<&Package, BuildError> {
    let root = metadata.root_package().map_or(
        Err(cargo_metadata::Error::CargoMetadata {
            stderr: "failed to find root project".to_string(),
        }),
        |p| Ok(p),
    )?;
    Ok(root)
}

pub fn with_root_crate_checked<F: FnMut(&Package) -> Result<(), BuildError>>(
    mut f: F,
) -> Result<(), BuildError> {
    with_root_crate(|root, metadata| {
        if needed_for_lisp_fn_export(root.id.clone(), &metadata) {
            f(root)?;
        }
        Ok(())
    })
}

pub fn with_root_crate<F: FnMut(&Package, &Metadata) -> Result<(), BuildError>>(
    mut f: F,
) -> Result<(), BuildError> {
    with_metadata(|metadata| {
        let root = root_crate(&metadata)?;
        f(root, &metadata)
    })
}

pub fn lisp_macro_crate(metadata: &Metadata) -> Option<&Package> {
    metadata
        .workspace_packages()
        .clone()
        .into_iter()
        .find(|p| p.id.repr.contains("lisp-macro"))
}

pub fn emacs_sys_crate(metadata: &Metadata) -> Option<&Package> {
    metadata
        .workspace_packages()
        .clone()
        .into_iter()
        .find(|p| p.id.repr.contains("emacs-sys"))
}

/// Find list of workspace members with lisp-macro in its direct dependency graph
pub fn with_enabled_crates<F: FnMut(Vec<&Package>) -> Result<(), BuildError>>(
    mut f: F,
) -> Result<(), BuildError> {
    with_metadata(|metadata| {
        let root_package = root_crate(&metadata)?;

        let packages = metadata.workspace_packages();

        let deps = resolve_deps(root_package.id.clone(), &metadata);

        let packages: Vec<&Package> = packages
            .into_iter()
            .filter(|p| {
                deps.iter().find(|id| p.id == **id).is_some()
                    && needed_for_c_export(p.id.clone(), &metadata)
            })
            .collect();

        f(packages)
    })
}

/// Find list of workspace members with lisp-macro in its dependency graph
pub fn with_enabled_crates_all<F: FnMut(Vec<&Package>) -> Result<(), BuildError>>(
    mut f: F,
) -> Result<(), BuildError> {
    with_metadata(|metadata| {
        let root_package = root_crate(&metadata)?;

        let packages = metadata.workspace_packages();
        let mut all_deps: Vec<PackageId> = Vec::new();
        resolve_deps_recursively(root_package.id.clone(), &mut all_deps, &metadata);

        let packages: Vec<&Package> = packages
            .into_iter()
            .filter(|p| {
                all_deps.iter().find(|id| p.id == **id).is_some()
                    && needed_for_globals(p.id.clone(), &metadata)
            })
            .collect();

        f(packages)
    })
}

pub fn available_features() -> Result<Vec<String>, BuildError> {
    let metadata = cargo_metadata::MetadataCommand::new().exec()?;

    let root_package = metadata.root_package();
    let features = root_package.map_or(Vec::new(), |r| r.features.clone().into_keys().collect());
    Ok(features)
}

pub fn enabled_features() -> Result<Vec<String>, BuildError> {
    fn is_set(name: &str) -> bool {
        std::env::var(name) == Ok("1".to_string())
    }

    let all_features = available_features()?;
    let mut features: Vec<String> = Vec::new();
    //TODO report upstream, CARGO_FEATURE_* is only set when using --features,
    // not for default features from Cargo.toml, not sure about `--no-default-features` `--all-features`
    for feature in all_features {
        let env_key = format!("CARGO_FEATURE_{}", feature.to_uppercase().replace("-", "_"));
        if is_set(env_key.as_str()) {
            features.push(feature);
        }
    }

    Ok(features)
}

pub fn package_targets(package: &Package) -> Vec<&cargo_metadata::Target> {
    package
        .targets
        .iter()
        .filter(|t| {
            t.kind.get(0).map_or(false, |k| match k.as_str() {
                "example" | "test" | "bench" | "custom-build" => false,
                _ => true,
            })
        })
        .collect()
}

pub fn target_source(t: &cargo_metadata::Target) -> Result<HashSet<PathBuf>, BuildError> {
    use cargo_files_core::get_target_files;
    use cargo_files_core::Target;
    let files = get_target_files(&Target::from_target(t))?;
    Ok(files)
}

pub fn package_source(p: &Package) -> HashSet<PathBuf> {
    let files = package_targets(p)
        .into_iter()
        .fold(HashSet::new(), |mut all_files, t| {
            match target_source(t) {
                Ok(files) => all_files.extend(files),
                Err(err) => eprintln!("Failed to get target source {err:?}"),
            };

            all_files
        });
    files
}

pub fn packages_source(packages: Vec<&Package>) -> HashSet<PathBuf> {
    packages.into_iter().fold(HashSet::new(), |mut acc, p| {
        let files = package_source(p);
        acc.extend(files);
        acc
    })
}
