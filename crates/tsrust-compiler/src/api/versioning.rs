//! Versioned wire baselines, independent of artifact fingerprints and frameworks.
use super::Manifest;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SdkReleaseIdentity {
    pub api_version: u32,
    pub contract_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApiRelease {
    pub api_version: u32,
    /// Retained legacy manifests may predate SDK release metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sdk_version: Option<String>,
    pub manifest: Manifest,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApiHistory {
    pub schema_version: u32,
    pub package_name: String,
    pub releases: Vec<ApiRelease>,
    /// Small immutable release references; unchanged DTO schemas are stored once.
    pub sdk_releases: BTreeMap<String, SdkReleaseIdentity>,
}

fn version(value: &str) -> Result<[u32; 3], String> {
    let parts = value.split('.').collect::<Vec<_>>();
    if parts.len() != 3 || parts.iter().any(|part| part.is_empty() || part.len() > 1 && part.starts_with('0') || !part.bytes().all(|byte| byte.is_ascii_digit())) {
        return Err("SDK version must be major.minor.patch with nonnegative integers".into());
    }
    let values = parts.into_iter().map(|part| part.parse::<u32>().map_err(|_| "SDK version component is out of range".to_owned())).collect::<Result<Vec<_>, _>>()?;
    Ok([values[0], values[1], values[2]])
}

impl ApiHistory {
    pub fn new(package_name: &str) -> Self {
        Self { schema_version: 1, package_name: package_name.into(), releases: Vec::new(), sdk_releases: BTreeMap::new() }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 || self.package_name.is_empty() { return Err("Invalid API compatibility history header".into()); }
        let mut identities = BTreeSet::new();
        let mut sdk_versions = BTreeSet::new();
        let mut last = None;
        for release in &self.releases {
            if release.api_version == 0 { return Err("API compatibility versions start at 1".into()); }
            release.manifest.validate()?;
            if !identities.insert((release.api_version, release.manifest.identity())) { return Err("Duplicate retained API contract".into()); }
            if let Some(sdk) = &release.sdk_version {
                let sdk = version(sdk)?;
                if !sdk_versions.insert(sdk) { return Err("One SDK version cannot identify different API releases".into()); }
                if last.is_some_and(|previous| sdk <= previous) { return Err("SDK history must be in increasing version order".into()); }
                last = Some(sdk);
                let expected = SdkReleaseIdentity { api_version: release.api_version, contract_id: release.manifest.wire_identity(release.api_version) };
                if self.sdk_releases.get(release.sdk_version.as_ref().unwrap()) != Some(&expected) { return Err("SDK release reference does not match its wire baseline".into()); }
            }
        }
        let mut latest = None;
        for (sdk, identity) in &self.sdk_releases {
            let sdk = version(sdk)?;
            if !self.releases.iter().any(|release| release.api_version == identity.api_version && release.manifest.wire_identity(release.api_version) == identity.contract_id) { return Err("SDK release refers to an unknown API baseline".into()); }
            latest = Some(latest.map_or(sdk, |previous: [u32; 3]| previous.max(sdk)));
        }
        if latest != last { return Err("SDK history does not retain its newest release".into()); }
        Ok(())
    }

    pub fn retain(&mut self, api_version: u32, manifest: Manifest) -> Result<(), String> {
        self.validate()?;
        if self.releases.iter().any(|release| release.api_version == api_version && release.manifest == manifest) { return Ok(()); }
        let release = ApiRelease { api_version, sdk_version: None, manifest };
        self.releases.push(release);
        if let Err(error) = self.validate() { self.releases.pop(); return Err(error); }
        Ok(())
    }

    /// Merge canonical/export history without losing newer published metadata.
    pub fn merge(&mut self, other: &Self) -> Result<(), String> {
        self.validate()?; other.validate()?;
        if self.package_name != other.package_name { return Err("Cannot merge API history from another package".into()); }
        let mut next = self.clone();
        for (sdk, identity) in &other.sdk_releases {
            if next.sdk_releases.get(sdk).is_some_and(|existing| existing != identity) { return Err("An SDK version identifies conflicting API releases".into()); }
            next.sdk_releases.insert(sdk.clone(), identity.clone());
        }
        for release in &other.releases {
            if let Some(existing) = next.releases.iter_mut().find(|existing| existing.api_version == release.api_version && existing.manifest == release.manifest) {
                if release.sdk_version.as_deref().map(version).transpose()? > existing.sdk_version.as_deref().map(version).transpose()? { existing.sdk_version = release.sdk_version.clone(); }
            } else { next.releases.push(release.clone()); }
        }
        // Stable ordering keeps explicit legacy baseline order, followed by
        // monotonic SDK releases. No dates or filesystem timestamps enter output.
        next.releases.sort_by_key(|release| release.sdk_version.as_deref().map(|sdk| version(sdk).expect("validated SDK version")));
        next.validate()?;
        *self = next;
        Ok(())
    }

    /// Compatibility is checked before returning a new immutable candidate.
    /// Generators must still preflight ownership/source identity before writing it.
    pub fn advance(&self, package_name: &str, sdk_version: &str, api_version: u32, manifest: &Manifest) -> Result<Self, String> {
        self.advance_impl(package_name, sdk_version, api_version, manifest, true)
    }

    pub fn advance_client(&self, package_name: &str, sdk_version: &str, api_version: u32, manifest: &Manifest) -> Result<Self, String> {
        self.advance_impl(package_name, sdk_version, api_version, manifest, false)
    }

    fn advance_impl(&self, package_name: &str, sdk_version: &str, api_version: u32, manifest: &Manifest, published_api: bool) -> Result<Self, String> {
        self.validate()?;
        manifest.validate()?;
        if self.package_name != package_name { return Err("API history belongs to another package; export to a separate output instead of replacing it".into()); }
        if api_version == 0 { return Err("API compatibility versions start at 1".into()); }
        let requested = version(sdk_version)?;
        let identity = SdkReleaseIdentity { api_version, contract_id: manifest.wire_identity(api_version) };
        if self.sdk_releases.get(sdk_version).is_some_and(|existing| existing != &identity) { return Err("API changed without an SDK version change; select a new SDK version".into()); }
        if self.releases.iter().any(|release| release.api_version > api_version) { return Err("The current API compatibility version cannot move backwards".into()); }
        if let Some(previous) = self.releases.iter().rev().find(|release| release.sdk_version.is_some()) {
            let released = version(previous.sdk_version.as_deref().unwrap())?;
            if requested < released { return Err("SDK version cannot move backwards".into()); }
            if requested == released {
                if previous.api_version != api_version || previous.manifest != *manifest { return Err("API changed without an SDK version change; select a new SDK version".into()); }
            }
            if api_version < previous.api_version { return Err("The current API compatibility version cannot move backwards".into()); }
            if api_version > previous.api_version && requested[0] <= released[0] { return Err("A new API compatibility version requires a new SDK major version".into()); }
        }
        for previous in self.releases.iter().filter(|release| release.api_version == api_version) {
            for old in &previous.manifest.operations {
                let Some(current) = manifest.operations.iter().find(|operation| operation.name == old.name) else {
                    if !published_api && previous.sdk_version.is_none() { continue; }
                    return Err(format!("API v{api_version} cannot remove operation {:?}", old.name));
                };
                if !current.supports_previous(old) {
                    return Err(format!("API v{api_version} operation {:?} breaks retained clients; preserve its contract or explicitly version the API", old.name));
                }
                if !current.accepts_previous_result(old) {
                    return Err(format!("API v{api_version} operation {:?} requires results older replicas cannot provide; make additions optional or explicitly version the API", old.name));
                }
            }
        }
        let mut next = self.clone();
        next.sdk_releases.insert(sdk_version.into(), identity);
        // Keep one baseline per wire shape, with its latest SDK version. This
        // preserves version monotonicity without copying unchanged schemas.
        next.releases.retain(|release| release.api_version != api_version || release.manifest != *manifest);
        next.releases.push(ApiRelease { api_version, sdk_version: Some(sdk_version.into()), manifest: manifest.clone() });
        next.releases.sort_by_key(|release| release.sdk_version.as_deref().map(|sdk| version(sdk).expect("validated SDK version")));
        next.validate()?;
        Ok(next)
    }
}
