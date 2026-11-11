//! `tsrust package`: a release build of a std:ui application, packaged for the
//! host platform. macOS gets a signed `.app` (optionally notarized and
//! stapled) as a zip for updates and a DMG for downloads; Windows a zip; Linux
//! a tarball and, when `dpkg-deb` exists, a `.deb`. Every artifact is listed in
//! `SHA256SUMS`, which the in-app updater verifies. `app.json` beside the
//! executable records the name, identifier, version and update repository.
use sha2::{Digest, Sha256};
use std::{fs, path::{Path, PathBuf}, process::Command};

const USAGE: &str = "usage: tsrust package [entry.tsrx] [--name NAME] [--id com.example.app] [--version 1.0.0] [--icon icon.png] [--repo owner/name] [--out dist] [--sign IDENTITY] [--notarize KEYCHAIN_PROFILE]. Defaults come from tsrust.config.json package; CLI options override them.";

#[derive(Default,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PackageConfig {
    pub name:Option<String>,pub id:Option<String>,pub version:Option<String>,pub icon:Option<PathBuf>,
    pub repo:Option<String>,pub out:Option<PathBuf>,pub sign:Option<String>,pub notarize:Option<String>,
}

struct Options {
    entry: PathBuf, name: String, id: String, version: String, icon: Option<PathBuf>,
    repo: String, out: PathBuf, sign: Option<String>, notarize: Option<String>,
}

fn options(args: impl Iterator<Item = String>) -> Result<Options, String> {
    options_for_project(args,super::project_config::discover()?)
}

fn options_for_project(args:impl Iterator<Item=String>, project:Option<super::project_config::Project>)->Result<Options,String> {
    let mut args = args.peekable();
    let (mut entry, config, default_out)=match project {Some(project)=>(Some(project.entry),project.package,project.root.join("dist")),None=>(None,PackageConfig::default(),PathBuf::from("dist"))};
    let (mut name, mut id, mut version, mut icon, mut repo, mut out, mut sign, mut notarize) = (config.name, config.id, config.version, config.icon, config.repo.unwrap_or_default(), config.out.unwrap_or(default_out), config.sign, config.notarize);
    let mut explicit_entry=false;
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value. {USAGE}"));
        match arg.as_str() {
            "--name" => name = Some(value()?),
            "--id" => id = Some(value()?),
            "--version" => version = Some(value()?),
            "--icon" => icon = Some(PathBuf::from(value()?)),
            "--repo" => repo = value()?,
            "--out" | "-o" => out = PathBuf::from(value()?),
            "--sign" => sign = Some(value()?),
            "--notarize" => notarize = Some(value()?),
            "-h" | "--help" => return Err(USAGE.into()),
            flag if flag.starts_with('-') => return Err(format!("Unknown option {flag}. {USAGE}")),
            path if !explicit_entry => {entry = Some(PathBuf::from(path));explicit_entry=true;},
            extra => return Err(format!("Unexpected argument {extra}. {USAGE}")),
        }
    }
    let entry = entry.ok_or_else(|| USAGE.to_owned())?;
    let stem = entry.file_stem().and_then(|stem| stem.to_str()).unwrap_or("App");
    let name = name.unwrap_or_else(|| {let mut chars = stem.chars(); chars.next().map(|first| first.to_uppercase().chain(chars).collect()).unwrap_or_else(|| "App".into())});
    if name.trim().is_empty() || matches!(name.as_str(),"."|"..") || name.chars().any(char::is_control) || name.contains(['/', '\\', ':']) {return Err(format!("Invalid application name {name:?}"));}
    let id = id.unwrap_or_else(|| format!("com.example.{}", name.to_lowercase().replace(|c: char| !c.is_ascii_alphanumeric(), "")));
    let version = version.unwrap_or_else(|| "0.1.0".into());
    if version.split('.').count() != 3 || version.split('.').any(|part| part.parse::<u64>().is_err()) {return Err(format!("Version must be MAJOR.MINOR.PATCH, got {version}"));}
    if notarize.is_some() && sign.is_none() {return Err("--notarize requires --sign".into());}
    if !repo.is_empty() && repo.split('/').count() != 2 {return Err(format!("--repo must be owner/name, got {repo}"));}
    Ok(Options {entry, name, id, version, icon, repo, out, sign, notarize})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn project()->super::super::project_config::Project {
        super::super::project_config::parse(Path::new("/project"),r#"{"schemaVersion":1,"name":"notes","build":{"entry":"src/main.tsrx","output":"target/notes"},"package":{"name":"Notes","id":"dev.notes","version":"1.2.3","icon":"assets/icon.png","out":"artifacts","repo":"owner/notes","sign":"Developer ID","notarize":"profile"}}"#).unwrap()
    }
    #[test]
    fn config_defaults_and_cli_overrides_use_their_own_path_roots() {
        let defaults=options_for_project(std::iter::empty(),Some(project())).unwrap();
        assert_eq!(defaults.entry,Path::new("/project/src/main.tsrx"));
        assert_eq!((defaults.name.as_str(),defaults.id.as_str(),defaults.version.as_str(),defaults.repo.as_str()),("Notes","dev.notes","1.2.3","owner/notes"));
        assert_eq!(defaults.icon.unwrap(),Path::new("/project/assets/icon.png"));
        assert_eq!(defaults.out,Path::new("/project/artifacts"));
        assert_eq!((defaults.sign.as_deref(),defaults.notarize.as_deref()),(Some("Developer ID"),Some("profile")));
        let override_=options_for_project(["other.tsrx","--name","Other","--id","dev.other","--version","2.0.0","--icon","other.png","--out","elsewhere","--repo","owner/other","--sign","Another ID","--notarize","another-profile"].map(str::to_owned).into_iter(),Some(project())).unwrap();
        assert_eq!(override_.entry,Path::new("other.tsrx"));
        assert_eq!(override_.icon.unwrap(),Path::new("other.png"));
        assert_eq!(override_.out,Path::new("elsewhere"));
        assert_eq!((override_.name.as_str(),override_.id.as_str(),override_.version.as_str(),override_.repo.as_str()),("Other","dev.other","2.0.0","owner/other"));
        assert_eq!((override_.sign.as_deref(),override_.notarize.as_deref()),(Some("Another ID"),Some("another-profile")));
        for package in [r#"{"unknown":true}"#,r#"{"version":1}"#,r#"{"out":"../escape"}"#,r#"{"icon":"/absolute.png"}"#] {
            let json=format!(r#"{{"schemaVersion":1,"name":"notes","build":{{"entry":"main.tsrx","output":"target/notes"}},"package":{package}}}"#);
            assert!(super::super::project_config::parse(Path::new("/project"),&json).is_err(),"{package}");
        }
        for arguments in [vec!["main.tsrx","--name",".."],vec!["main.tsrx","--version","1.2"],vec!["main.tsrx","--notarize","profile"],vec!["main.tsrx","other.tsrx"]] {
            assert!(options_for_project(arguments.into_iter().map(str::to_owned),None).is_err());
        }
    }
}

fn run_tool(command: &mut Command, what: &str) -> Result<(), String> {
    let status = command.status().map_err(|error| format!("Cannot run {what}: {error}"))?;
    if status.success() {Ok(())} else {Err(format!("{what} failed ({status})"))}
}

fn platform() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "macos") {"macos"} else if cfg!(windows) {"windows"} else {"linux"};
    let arch = if cfg!(target_arch = "aarch64") {"arm64"} else {"x64"};
    (os, arch)
}

pub(crate) fn run(args: impl Iterator<Item = String>) -> Result<u8, String> {
    let options = options(args)?;
    fs::create_dir_all(&options.out).map_err(|error| format!("Cannot create {}: {error}", options.out.display()))?;
    let work = options.out.join(".package");
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).map_err(|error| error.to_string())?;
    let executable = work.join(if cfg!(windows) {format!("{}.exe", options.name)} else {options.name.clone()});
    let compiler = std::env::current_exe().map_err(|error| error.to_string())?;
    run_tool(Command::new(compiler).arg("build").arg(&options.entry).arg("--release").arg("-o").arg(&executable), "tsrust build --release")?;
    let manifest = serde_json::json!({"name": options.name, "id": options.id, "version": options.version, "repo": options.repo});
    let manifest = serde_json::to_string_pretty(&manifest).map_err(|error| error.to_string())?;
    let (os, arch) = platform();
    // GitHub rewrites spaces in asset names, so file names use dashes; the updater matches this.
    let base = format!("{}-{}-{os}-{arch}", options.name.split_whitespace().collect::<Vec<_>>().join("-"), options.version);
    let artifacts = match os {
        "macos" => macos(&options, &executable, &manifest, &work, &base)?,
        "windows" => vec![portable_zip(&options, &executable, &manifest, &work, &base)?],
        _ => linux(&options, &executable, &manifest, &work, &base)?,
    };
    let mut sums = String::new();
    for artifact in &artifacts {
        let bytes = fs::read(artifact).map_err(|error| error.to_string())?;
        let digest = Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        sums.push_str(&format!("{digest}  {}\n", artifact.file_name().unwrap().to_string_lossy()));
    }
    fs::write(options.out.join("SHA256SUMS"), sums).map_err(|error| error.to_string())?;
    let _ = fs::remove_dir_all(&work);
    for artifact in &artifacts {println!("Packaged {}", artifact.display());}
    println!("Packaged {}", options.out.join("SHA256SUMS").display());
    Ok(0)
}

fn plist(options: &Options) -> String {
    let escape = |text: &str| text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key><string>{name}</string>
  <key>CFBundleIdentifier</key><string>{id}</string>
  <key>CFBundleName</key><string>{name}</string>
  <key>CFBundleDisplayName</key><string>{name}</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>{version}</string>
  <key>CFBundleVersion</key><string>{version}</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>LSMinimumSystemVersion</key><string>12.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSSupportsAutomaticGraphicsSwitching</key><true/>
  <key>NSMicrophoneUsageDescription</key><string>{name} uses the microphone for dictation.</string>
  <key>NSSpeechRecognitionUsageDescription</key><string>{name} turns your speech into text.</string>
</dict>
</plist>
"#, name = escape(&options.name), id = escape(&options.id), version = escape(&options.version))
}

/// A PNG becomes AppIcon.icns through sips and iconutil (both part of macOS).
fn icns(png: &Path, resources: &Path, work: &Path) -> Result<(), String> {
    let set = work.join("AppIcon.iconset");
    fs::create_dir_all(&set).map_err(|error| error.to_string())?;
    for size in [16, 32, 128, 256, 512] {
        for scale in [1, 2] {
            let file = if scale == 1 {format!("icon_{size}x{size}.png")} else {format!("icon_{size}x{size}@2x.png")};
            let pixels = (size * scale).to_string();
            run_tool(Command::new("sips").args(["-z", &pixels, &pixels]).arg(png).arg("--out").arg(set.join(file)).stdout(std::process::Stdio::null()), "sips")?;
        }
    }
    run_tool(Command::new("iconutil").args(["-c", "icns"]).arg(&set).arg("-o").arg(resources.join("AppIcon.icns")), "iconutil")
}

fn macos(options: &Options, executable: &Path, manifest: &str, work: &Path, base: &str) -> Result<Vec<PathBuf>, String> {
    let app = work.join(format!("{}.app", options.name));
    let (contents, macos, resources) = (app.join("Contents"), app.join("Contents/MacOS"), app.join("Contents/Resources"));
    for directory in [&macos, &resources] {fs::create_dir_all(directory).map_err(|error| error.to_string())?;}
    fs::copy(executable, macos.join(&options.name)).map_err(|error| error.to_string())?;
    fs::write(contents.join("Info.plist"), plist(options)).map_err(|error| error.to_string())?;
    fs::write(resources.join("app.json"), manifest).map_err(|error| error.to_string())?;
    if let Some(icon) = &options.icon {icns(icon, &resources, work)?;}
    let zip = options.out.join(format!("{base}.zip"));
    let dmg = options.out.join(format!("{base}.dmg"));
    let make_zip = |zip: &Path| {let _ = fs::remove_file(zip); run_tool(Command::new("ditto").args(["-c", "-k", "--keepParent"]).arg(&app).arg(zip), "ditto")};
    if let Some(identity) = &options.sign {
        // Hardened runtime and a secure timestamp are required for notarization.
        run_tool(Command::new("codesign").args(["--force", "--options", "runtime", "--timestamp", "--sign", identity]).arg(&app), "codesign")?;
        run_tool(Command::new("codesign").args(["--verify", "--strict", "--verbose=2"]).arg(&app), "codesign --verify")?;
    }
    make_zip(&zip)?;
    if let Some(profile) = &options.notarize {
        run_tool(Command::new("xcrun").args(["notarytool", "submit"]).arg(&zip).args(["--keychain-profile", profile, "--wait"]), "notarytool submit (app)")?;
        run_tool(Command::new("xcrun").args(["stapler", "staple"]).arg(&app), "stapler staple (app)")?;
        make_zip(&zip)?;
    }
    let _ = fs::remove_file(&dmg);
    run_tool(Command::new("hdiutil").args(["create", "-volname", &options.name, "-srcfolder"]).arg(&app).args(["-ov", "-format", "UDZO"]).arg(&dmg).stdout(std::process::Stdio::null()), "hdiutil create")?;
    if let Some(identity) = &options.sign {
        run_tool(Command::new("codesign").args(["--force", "--timestamp", "--sign", identity]).arg(&dmg), "codesign (dmg)")?;
    }
    if let Some(profile) = &options.notarize {
        run_tool(Command::new("xcrun").args(["notarytool", "submit"]).arg(&dmg).args(["--keychain-profile", profile, "--wait"]), "notarytool submit (dmg)")?;
        run_tool(Command::new("xcrun").args(["stapler", "staple"]).arg(&dmg), "stapler staple (dmg)")?;
    }
    Ok(vec![zip, dmg])
}

/// Windows: the executable and app.json in a zip (`tar` ships with Windows 10+).
fn portable_zip(options: &Options, executable: &Path, manifest: &str, work: &Path, base: &str) -> Result<PathBuf, String> {
    let folder = work.join(&options.name);
    fs::create_dir_all(&folder).map_err(|error| error.to_string())?;
    fs::copy(executable, folder.join(executable.file_name().unwrap())).map_err(|error| error.to_string())?;
    fs::write(folder.join("app.json"), manifest).map_err(|error| error.to_string())?;
    let zip = options.out.join(format!("{base}.zip"));
    let _ = fs::remove_file(&zip);
    let target = fs::canonicalize(&options.out).map_err(|error| error.to_string())?.join(format!("{base}.zip"));
    run_tool(Command::new("tar").arg("-a").arg("-cf").arg(&target).arg("-C").arg(work).arg(&options.name), "tar (zip)")?;
    Ok(zip)
}

fn linux(options: &Options, executable: &Path, manifest: &str, work: &Path, base: &str) -> Result<Vec<PathBuf>, String> {
    let folder = work.join(&options.name);
    fs::create_dir_all(&folder).map_err(|error| error.to_string())?;
    fs::copy(executable, folder.join(&options.name)).map_err(|error| error.to_string())?;
    fs::write(folder.join("app.json"), manifest).map_err(|error| error.to_string())?;
    let tarball = options.out.join(format!("{base}.tar.gz"));
    run_tool(Command::new("tar").arg("-czf").arg(&tarball).arg("-C").arg(work).arg(&options.name), "tar")?;
    let mut artifacts = vec![tarball];
    if Command::new("dpkg-deb").arg("--version").output().is_ok_and(|output| output.status.success()) {
        let package = options.name.to_lowercase().replace(|c: char| !c.is_ascii_alphanumeric(), "-");
        let root = work.join("deb");
        let (bin, share, apps, control) = (root.join("usr/bin"), root.join(format!("usr/share/{package}")), root.join("usr/share/applications"), root.join("DEBIAN"));
        for directory in [&bin, &share, &apps, &control] {fs::create_dir_all(directory).map_err(|error| error.to_string())?;}
        fs::copy(executable, share.join(&options.name)).map_err(|error| error.to_string())?;
        fs::write(share.join("app.json"), manifest).map_err(|error| error.to_string())?;
        fs::write(bin.join(&package), format!("#!/bin/sh\nexec /usr/share/{package}/{} \"$@\"\n", options.name)).map_err(|error| error.to_string())?;
        #[cfg(unix)] {use std::os::unix::fs::PermissionsExt; fs::set_permissions(bin.join(&package), fs::Permissions::from_mode(0o755)).map_err(|error| error.to_string())?;}
        fs::write(apps.join(format!("{package}.desktop")), format!("[Desktop Entry]\nType=Application\nName={}\nExec={package}\nCategories=Utility;\n", options.name)).map_err(|error| error.to_string())?;
        let architecture = if cfg!(target_arch = "aarch64") {"arm64"} else {"amd64"};
        fs::write(control.join("control"), format!("Package: {package}\nVersion: {}\nArchitecture: {architecture}\nMaintainer: {} maintainers\nDescription: {}\n", options.version, options.name, options.name)).map_err(|error| error.to_string())?;
        let deb = options.out.join(format!("{base}.deb"));
        run_tool(Command::new("dpkg-deb").args(["--build", "--root-owner-group"]).arg(&root).arg(&deb), "dpkg-deb")?;
        artifacts.push(deb);
    }
    Ok(artifacts)
}
