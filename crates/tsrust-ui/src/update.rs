//! Application updates from GitHub releases, for apps packaged by `tsrust package`.
//! `app.json` beside the executable (inside Contents/Resources on macOS) records
//! the name, version and repository. A check reads the repository's latest
//! release; an install downloads this platform's archive, verifies it against
//! the release's SHA256SUMS (and, for a signed macOS app, the signing team),
//! then swaps it into place. The new version runs after `relaunch`.
//! Downloads use the system curl, built into macOS, Windows 10+ and Linux desktops.
#![cfg_attr(not(feature = "gpui"), allow(dead_code))]
use sha2::{Digest, Sha256};
use std::{fs, path::{Path, PathBuf}, process::Command};

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct Manifest {#[serde(default)] name: String, #[serde(default)] id: String, #[serde(default)] version: String, #[serde(default)] repo: String}

/// The installed unit: a macOS `.app` bundle, or the folder holding the executable.
fn installation() -> Result<(PathBuf, PathBuf), String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let folder = executable.parent().ok_or("The executable has no folder")?.to_owned();
    if cfg!(target_os = "macos") && folder.ends_with("Contents/MacOS") {
        let bundle = folder.parent().and_then(Path::parent).ok_or("Invalid app bundle")?.to_owned();
        return Ok((bundle.join("Contents/Resources/app.json"), bundle));
    }
    Ok((folder.join("app.json"), folder))
}

fn manifest() -> Manifest {
    installation().ok().and_then(|(path, _)| fs::read_to_string(path).ok()).and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
}

/// The packaged app's name, id, version and update repository as JSON; empty fields when unpackaged.
pub fn app_info() -> String {serde_json::to_string(&manifest()).unwrap_or_default()}

fn version(text: &str) -> Option<(u64, u64, u64)> {
    let mut parts = text.trim().trim_start_matches('v').split(['.', '-', '+']).map(str::parse::<u64>);
    Some((parts.next()?.ok()?, parts.next()?.ok()?, parts.next()?.ok()?))
}

fn platform() -> (&'static str, &'static str, &'static str) {
    let os = if cfg!(target_os = "macos") {"macos"} else if cfg!(windows) {"windows"} else {"linux"};
    (os, if cfg!(target_arch = "aarch64") {"arm64"} else {"x64"}, if os == "linux" {"tar.gz"} else {"zip"})
}

/// This platform's archive name, matching `tsrust package` output (spaces become dashes).
fn asset_name(name: &str, version: &str) -> String {
    let (os, arch, extension) = platform();
    format!("{}-{version}-{os}-{arch}.{extension}", name.split_whitespace().collect::<Vec<_>>().join("-"))
}

/// The digest listed for `file` in a SHA256SUMS text.
fn listed_digest<'a>(sums: &'a str, file: &str) -> Option<&'a str> {
    sums.lines().find_map(|line| {let (digest, name) = line.split_once(char::is_whitespace)?; (name.trim_start().trim_start_matches('*') == file).then_some(digest)})
}

fn curl(url: &str, output: Option<&Path>, api: bool) -> Result<Vec<u8>, String> {
    let mut command = Command::new("curl");
    command.args(["-fsSL", "--proto", "=https", "--proto-redir", "=https", "--max-time", "900", "-H", "User-Agent: tsrust-updater"]);
    if api {command.args(["-H", "Accept: application/vnd.github+json"]);}
    if let Some(output) = output {command.arg("-o").arg(output);}
    let result = command.arg(url).output().map_err(|error| format!("Cannot run curl: {error}"))?;
    if result.status.success() {Ok(result.stdout)} else {Err(format!("Download failed: {}", String::from_utf8_lossy(&result.stderr).trim()))}
}

#[derive(serde::Deserialize)]
struct Asset {name: String, browser_download_url: String}
#[derive(serde::Deserialize)]
struct Release {tag_name: String, #[serde(default)] body: Option<String>, #[serde(default)] assets: Vec<Asset>}

fn latest(manifest: &Manifest) -> Result<Release, String> {
    if manifest.repo.is_empty() {return Err("This app has no update repository; package it with --repo owner/name".into());}
    let body = curl(&format!("https://api.github.com/repos/{}/releases/latest", manifest.repo), None, true)?;
    serde_json::from_slice(&body).map_err(|error| format!("Unexpected release response: {error}"))
}

/// Check for a newer release. Always JSON: current, latest, available, notes and error.
pub fn check() -> String {
    let manifest = manifest();
    let result = latest(&manifest).and_then(|release| {
        let current = version(&manifest.version).ok_or_else(|| format!("Invalid installed version {:?}", manifest.version))?;
        let next = version(&release.tag_name).ok_or_else(|| format!("Invalid release tag {:?}", release.tag_name))?;
        let name = release.tag_name.trim_start_matches('v').to_owned();
        let shipped = release.assets.iter().any(|asset| asset.name == asset_name(&manifest.name, &name));
        Ok(serde_json::json!({"latest": name, "available": next > current && shipped, "notes": release.body.unwrap_or_default()}))
    });
    let mut value = result.unwrap_or_else(|error| serde_json::json!({"latest": "", "available": false, "notes": "", "error": error}));
    value["current"] = manifest.version.into();
    if value.get("error").is_none() {value["error"] = "".into();}
    value.to_string()
}

fn run(command: &mut Command, what: &str) -> Result<(), String> {
    let output = command.output().map_err(|error| format!("Cannot run {what}: {error}"))?;
    if output.status.success() {Ok(())} else {Err(format!("{what} failed: {}", String::from_utf8_lossy(&output.stderr).trim()))}
}

/// The signing team of a macOS bundle, or None when unsigned or ad hoc.
fn team(bundle: &Path) -> Option<String> {
    let output = Command::new("codesign").args(["-dv", "--verbose=2"]).arg(bundle).output().ok()?;
    String::from_utf8_lossy(&output.stderr).lines().find_map(|line| line.strip_prefix("TeamIdentifier=")).filter(|team| *team != "not set").map(str::to_owned)
}

/// Move `next` into `target`, keeping the old copy until the move succeeds.
fn replace(next: &Path, target: &Path) -> Result<(), String> {
    let old = target.with_file_name(format!(".{}.old", target.file_name().unwrap_or_default().to_string_lossy()));
    let _ = fs::remove_dir_all(&old).or_else(|_| fs::remove_file(&old));
    let replaced = target.exists();
    if replaced {fs::rename(target, &old).map_err(|error| format!("Cannot replace {}: {error}", target.display()))?;}
    if let Err(error) = fs::rename(next, target) {
        if replaced {let _ = fs::rename(&old, target);}
        return Err(format!("Cannot install {}: {error}", target.display()));
    }
    // A running Windows executable cannot be deleted; the next install clears it.
    let _ = fs::remove_dir_all(&old).or_else(|_| fs::remove_file(&old));
    Ok(())
}

fn install_latest() -> Result<String, String> {
    let manifest = manifest();
    let release = latest(&manifest)?;
    let name = release.tag_name.trim_start_matches('v').to_owned();
    if version(&name) <= version(&manifest.version) {return Err(format!("{} {} is already the latest version", manifest.name, manifest.version));}
    let wanted = asset_name(&manifest.name, &name);
    let url = |file: &str| release.assets.iter().find(|asset| asset.name == file).map(|asset| asset.browser_download_url.clone()).ok_or_else(|| format!("The {} release has no {file}", release.tag_name));
    let (archive_url, sums_url) = (url(&wanted)?, url("SHA256SUMS")?);
    let (_, installed) = installation()?;
    let parent = installed.parent().ok_or("The installation has no parent folder")?;
    // Stage beside the installation so the final swap is a same-volume rename.
    let staging = parent.join(format!(".{}-update", manifest.name));
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging).map_err(|error| format!("Cannot write beside {}: {error}", installed.display()))?;
    let result = apply(&manifest.name, &wanted, &sums_url, &archive_url, &installed, &staging);
    let _ = fs::remove_dir_all(&staging);
    result.map(|_| name)
}

/// Download into `staging`, verify, unpack and swap over `installed`.
fn apply(name: &str, wanted: &str, sums_url: &str, archive_url: &str, installed: &Path, staging: &Path) -> Result<(), String> {
    let archive = staging.join(wanted);
    let sums = String::from_utf8_lossy(&curl(sums_url, None, false)?).into_owned();
    curl(archive_url, Some(&archive), false)?;
    let expected = listed_digest(&sums, wanted).ok_or_else(|| format!("SHA256SUMS does not list {wanted}"))?;
    let actual = Sha256::digest(fs::read(&archive).map_err(|error| error.to_string())?).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    if !actual.eq_ignore_ascii_case(expected) {return Err(format!("{wanted} does not match its SHA256SUMS digest"));}
    let unpacked = staging.join("unpacked");
    fs::create_dir_all(&unpacked).map_err(|error| error.to_string())?;
    if cfg!(target_os = "macos") {
        run(Command::new("ditto").args(["-x", "-k"]).arg(&archive).arg(&unpacked), "ditto")?;
        let bundle = unpacked.join(format!("{name}.app"));
        if let Some(team) = team(installed) {
            run(Command::new("codesign").args(["--verify", "--deep", "--strict"]).arg(&bundle), "codesign --verify")?;
            if self::team(&bundle).as_deref() != Some(team.as_str()) {return Err("The update is not signed by the same developer".into());}
        }
        replace(&bundle, installed)
    } else {
        let flags = if cfg!(windows) {"-xf"} else {"-xzf"};
        run(Command::new("tar").arg(flags).arg(&archive).arg("-C").arg(&unpacked), "tar")?;
        let folder = unpacked.join(name);
        for entry in fs::read_dir(&folder).map_err(|error| format!("The archive has no {name} folder: {error}"))? {
            let entry = entry.map_err(|error| error.to_string())?;
            replace(&entry.path(), &installed.join(entry.file_name()))?;
        }
        Ok(())
    }
}

/// Download, verify and install the latest release. Returns an empty string on success, else the error.
pub fn install() -> String {install_latest().map(|_| String::new()).unwrap_or_else(|error| error)}

/// Start the installed app again (the updated version after an install).
pub(crate) fn relaunch() -> Result<(), String> {
    let (_, installed) = installation()?;
    let spawned = if cfg!(target_os = "macos") && installed.extension().is_some_and(|extension| extension == "app") {
        Command::new("open").arg("-n").arg(&installed).spawn()
    } else {
        Command::new(std::env::current_exe().map_err(|error| error.to_string())?).args(std::env::args_os().skip(1)).spawn()
    };
    spawned.map(drop).map_err(|error| format!("Cannot relaunch: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn versions_assets_and_digests_follow_the_package_layout() {
        assert!(version("v1.10.0") > version("1.9.3"));
        assert_eq!(version("2.0.1-beta"), Some((2, 0, 1)));
        assert_eq!(version("next"), None);
        let (os, arch, extension) = platform();
        assert_eq!(asset_name("Demo", "1.2.3"), format!("Demo-1.2.3-{os}-{arch}.{extension}"));
        assert_eq!(asset_name("My  App", "1.0.0"), format!("My-App-1.0.0-{os}-{arch}.{extension}"));
        let sums = "abc123  Demo-1.2.3-macos-arm64.zip\ndef456 *Demo-1.2.3-windows-x64.zip\n";
        assert_eq!(listed_digest(sums, "Demo-1.2.3-macos-arm64.zip"), Some("abc123"));
        assert_eq!(listed_digest(sums, "Demo-1.2.3-windows-x64.zip"), Some("def456"));
        assert_eq!(listed_digest(sums, "Demo-1.2.3-linux-x64.tar.gz"), None);
        let unpackaged: serde_json::Value = serde_json::from_str(&check()).unwrap();
        assert_eq!(unpackaged["available"], false);
        assert!(unpackaged["error"].as_str().unwrap().contains("--repo"));
    }

    #[test]
    fn replace_swaps_files_and_folders() {
        let root = std::env::temp_dir().join(format!("tsrust-update-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("next.app")).unwrap();
        fs::write(root.join("next.app/marker"), "new").unwrap();
        fs::create_dir_all(root.join("App.app")).unwrap();
        fs::write(root.join("App.app/marker"), "old").unwrap();
        replace(&root.join("next.app"), &root.join("App.app")).unwrap();
        assert_eq!(fs::read_to_string(root.join("App.app/marker")).unwrap(), "new");
        assert!(!root.join(".App.app.old").exists());
        fs::write(root.join("next"), "2").unwrap();
        replace(&root.join("next"), &root.join("tool")).unwrap();
        assert_eq!(fs::read_to_string(root.join("tool")).unwrap(), "2");
        fs::remove_dir_all(&root).unwrap();
    }
}
