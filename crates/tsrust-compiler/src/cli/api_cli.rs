//! Export plain checked contracts without adopting or rewriting frontend tooling.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::{Component, Path, PathBuf}};
use tsrust_compiler::{api::{Manifest, Selection}, project::{FileSourceProvider, SourceProvider, load_project, check_graph}};

const CONFIG: &str = "tsrust.api.json";
const MARKER: &str = ".tsrust-api.json";

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Config { schema_version: u32, entry: String, output: String, package: Package, operations: Vec<Selection> }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Package { name: String, version: String }
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Marker { schema_version: u32, files: BTreeMap<String, String> }

fn relative(value: &str) -> Result<&Path, String> {
    let path = Path::new(value);
    if value.is_empty() || value.contains('\\') || path.components().any(|part| !matches!(part, Component::Normal(_))) {
        return Err(format!("API configuration path {value:?} must be relative without '.', '..' or backslashes"));
    }
    Ok(path)
}

fn hash(bytes: &[u8]) -> String { Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect() }

fn regular(path: &Path) -> Result<Option<Vec<u8>>, String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => fs::read(path).map(Some).map_err(|error| format!("{}: {error}", path.display())),
        Ok(_) => Err(format!("{} must be a regular file; generated output never follows links", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("{}: {error}", path.display())),
    }
}

fn safe_output(path: &Path) -> Result<(), String> {
    for parent in path.ancestors() {
        match fs::symlink_metadata(parent) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => return Err(format!("Output directory {} must not be a link or file", parent.display())),
            Ok(_) => {},
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
            Err(error) => return Err(format!("{}: {error}", parent.display())),
        }
    }
    Ok(())
}

fn atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let temporary = path.with_file_name(format!(".{}.tsrust-{}.tmp", path.file_name().unwrap().to_string_lossy(), std::process::id()));
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&temporary).map_err(|error| error.to_string())?;
    let result = (|| {
        file.write_all(bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        drop(file);
        fs::rename(&temporary, path).map_err(|error| error.to_string())
    })();
    if result.is_err() { let _ = fs::remove_file(&temporary); }
    result
}

struct OutputLock(PathBuf);
impl Drop for OutputLock { fn drop(&mut self) { let _ = fs::remove_file(&self.0); } }

fn write_package(output: &Path, files: &BTreeMap<String, String>, check: bool) -> Result<(), String> {
    safe_output(output)?;
    let _lock = if check { None } else {
        fs::create_dir_all(output).map_err(|error| error.to_string())?;
        let lock = output.join(".tsrust-api.lock");
        fs::OpenOptions::new().write(true).create_new(true).open(&lock).map_err(|error| format!("Cannot lock API output (another generator or stale lock may exist): {error}"))?;
        Some(OutputLock(lock))
    };
    let previous = regular(&output.join(MARKER))?.map(|bytes| serde_json::from_slice::<Marker>(&bytes).map_err(|error| format!("Invalid generated API marker: {error}"))).transpose()?;
    if previous.as_ref().is_some_and(|marker| marker.schema_version != 1) { return Err("Unsupported generated API marker version".into()); }
    if previous.is_none() && output.exists() && fs::read_dir(output).map_err(|error| error.to_string())?
        .any(|entry| entry.map_or(true, |entry| entry.file_name() != ".tsrust-api.lock")) {
        return Err("API output must be empty or previously generated; existing application files are preserved".into());
    }
    let next = Marker { schema_version: 1, files: files.iter().map(|(name, contents)| (name.clone(), hash(contents.as_bytes()))).collect() };
    for (name, contents) in files {
        let current = regular(&output.join(name))?;
        if check {
            if current.as_deref() != Some(contents.as_bytes()) { return Err(format!("Generated API is stale: {name}; run tsrust api generate")); }
        } else if let Some(bytes) = current {
            let current = hash(&bytes);
            if current != next.files[name] && previous.as_ref().and_then(|marker| marker.files.get(name)) != Some(&current) {
                return Err(format!("Generated API file {name} has user edits; preserve or reconcile them before regenerating"));
            }
        }
    }
    if check {
        if previous.as_ref().map(|marker| &marker.files) != Some(&next.files) { return Err("Generated API marker is stale; run tsrust api generate".into()); }
        return Ok(());
    }
    // All contracts and owner-edit checks complete before the first replacement.
    // A failed partial write is recoverable: preflight accepts old or planned bytes.
    for (name, contents) in files {
        if regular(&output.join(name))?.as_deref() != Some(contents.as_bytes()) { atomic(&output.join(name), contents.as_bytes())?; }
    }
    atomic(&output.join(MARKER), format!("{}\n", serde_json::to_string_pretty(&next).map_err(|error| error.to_string())?).as_bytes())
}

pub(super) fn run(args: impl Iterator<Item = String>) -> Result<u8, String> {
    let mut args = args;
    if args.next().as_deref() != Some("generate") { return Err("Usage: tsrust api generate [--config tsrust.api.json] [--out directory] [--check]".into()); }
    let mut config = None;
    let mut output = None;
    let mut check = false;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--config" if config.is_none() => config = Some(PathBuf::from(args.next().ok_or("--config requires a file")?)),
            "--out" if output.is_none() => output = Some(PathBuf::from(args.next().ok_or("--out requires a directory")?)),
            "--check" if !check => check = true,
            _ => return Err(format!("Unknown or repeated API generation option: {argument}")),
        }
    }
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    let path = if let Some(config) = config { cwd.join(config) } else {
        cwd.ancestors().map(|root| root.join(CONFIG)).find(|path| path.is_file()).ok_or("No tsrust.api.json found; pass --config for an existing project")?
    };
    let root = path.parent().ok_or("API config needs a parent directory")?;
    let config: Config = serde_json::from_slice(&fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?).map_err(|error| format!("{}: {error}", path.display()))?;
    if config.schema_version != 1 { return Err("API configuration schemaVersion must be 1".into()); }
    let entry = root.join(relative(&config.entry)?);
    let output = output.map(|path| cwd.join(path)).unwrap_or(root.join(relative(&config.output)?));
    let mut selections = config.operations;
    for selection in &mut selections {
        let path = root.join(relative(&selection.module)?);
        selection.module = FileSourceProvider.resolve(None, path.to_str().ok_or("API source path must be UTF-8")?)?.0;
    }
    let graph = load_project(&FileSourceProvider, entry.to_str().ok_or("API entry must be UTF-8")?).map_err(|error| error.message)?;
    let project = check_graph(graph).map_err(|error| error.message)?;
    let manifest = Manifest::from_project(&project, &selections)?;
    let files = manifest.client_package(&config.package.name, &config.package.version)?;
    write_package(&output, &files, check)?;
    println!("{} API client: {} operations, contract {}", if check { "Verified" } else { "Generated" }, manifest.operations.len(), manifest.identity());
    Ok(0)
}

