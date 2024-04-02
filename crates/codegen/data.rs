use std::collections::HashSet;
use std::path::PathBuf;

use super::error::BuildError;
use cargo_metadata::CargoOpt;
use cargo_metadata::Metadata;
pub use cargo_metadata::Package;
use cargo_metadata::PackageId;

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

pub fn with_root_crate<F: FnMut(&Package) -> Result<(), BuildError>>(
    mut f: F,
) -> Result<(), BuildError> {
    with_metadata(|metadata| {
        let root_package = root_crate(&metadata)?;
        f(root_package)
    })
}

/// Find list of workspace members with lisp-macro in its dependencies
pub fn with_enabled_crates<F: FnMut(Vec<&Package>) -> Result<(), BuildError>>(
    mut f: F,
) -> Result<(), BuildError> {
    with_metadata(|metadata| {
        let root_package = root_crate(&metadata)?;

        let packages = metadata.workspace_packages();
        let lisp_macro_package = packages
            .clone()
            .into_iter()
            .find(|p| p.id.repr.contains("lisp-macro"));

        let deps = metadata
            .resolve
            .as_ref()
            .and_then(|r| {
                r.nodes.iter().find(|n| root_package.id == n.id)
                // fixme needs to find nest dep
            })
            .map(|n| {
                n.dependencies
                    .iter()
                    .filter(|d| d.repr.starts_with("path+file:///"))
                    .collect::<Vec<&PackageId>>()
            })
            .map_or(Vec::new(), |ids| ids);

        let packages: Vec<&Package> = packages
            .into_iter()
            .filter(|p| {
                deps.iter().find(|id| p.id == ***id).is_some()
                    && p.dependencies
                        .iter()
                        .find(|d| {
                            d.path.is_some()
                                && lisp_macro_package.map_or(false, |r| r.name == d.name)
                        })
                        .is_some()
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
        println!("cargo:rerun-if-env-changed={}", env_key);
        if is_set(env_key.as_str()) {
            features.push(feature);
        }
    }

    Ok(features)
}

pub fn packages_source(packages: Vec<&Package>) -> HashSet<PathBuf> {
    use cargo_files_core::get_target_files;
    use cargo_files_core::Target;

    packages
        .into_iter()
        .fold(Vec::new(), |mut acc, p| {
            acc.append(&mut p.targets.clone());
            acc
        })
        .iter()
        .filter(|t| {
            t.kind.get(0).map_or(false, |k| match k.as_str() {
                "example" | "test" | "bench" | "custom-build" => false,
                _ => true,
            })
        })
        .fold(HashSet::new(), |mut all_files, t| {
            match get_target_files(&Target::from_target(t)) {
                Ok(files) => all_files.extend(files),
                _ => {}
            };

            all_files
        })
}
