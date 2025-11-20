use super::Flags;
use super::License;
use super::Maintainers;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use url::Url;

// See:
// https://devmanual.gentoo.org/general-concepts/dependencies/
// https://projects.gentoo.org/pms/8/pms.html#x1-410005

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageCategory {
    name: String,
    maintainers: Option<Maintainers>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Slot {
    versions: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageVersion {}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageDependency {
    package: Package,
    strict_version: Option<PackageVersion>,
    min_version: Option<PackageVersion>,
    max_version: Option<PackageVersion>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    name: String,
    category: PackageCategory,
    tags: Vec<String>,
    url: Option<Url>,
    source: Option<Url>,
    maintainers: Maintainers,
    version: String,
    flags: Flags,
    slot: Slot,
    // build: Option<BuildConfig>,
    license: License,
    dep_licenses: Vec<License>,
    keywords: Vec<String>,
    binaries: Vec<String>,
    dependencies: Vec<PackageDependency>,
    env_vars: BTreeMap<String, String>,
}
