//! Export plain checked contracts without adopting or rewriting frontend tooling.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::{Component, Path, PathBuf}};
use tsrust_compiler::{api::{CheckedApi, Manifest, NativeImport, NativeServer, Selection}, project::{SourceId, SourceProvider, load_project, check_graph}};
use super::package_validation::{Candidate, normalize};

const CONFIG: &str = "tsrust.api.json";
const MARKER: &str = ".tsrust-api.json";

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Config { schema_version: u32, entry: String, output: String, package: Package, operations: Vec<Selection>, #[serde(default)] server: Option<Server> }
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Server {
    output: String, context: Import, authorize: Import,
    #[serde(default)] compatible: Vec<String>,
    #[serde(default = "byte_limit")] max_request_bytes: u32,
    #[serde(default = "byte_limit")] max_response_bytes: u32,
}
fn byte_limit() -> u32 { 65536 }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Import { module: String, export: String }
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

struct OutputPlan { output: PathBuf, files: BTreeMap<String, String>, next: Marker, observed: BTreeMap<String, Option<String>>, _lock: Option<OutputLock> }
fn prepare_package(output: &Path, files: BTreeMap<String, String>, check: bool) -> Result<OutputPlan, String> {
    safe_output(output)?;
    let lock = if check { None } else {
        fs::create_dir_all(output).map_err(|error| error.to_string())?;
        let lock = output.join(".tsrust-api.lock");
        fs::OpenOptions::new().write(true).create_new(true).open(&lock).map_err(|error| format!("Cannot lock API output (another generator or stale lock may exist): {error}"))?;
        Some(OutputLock(lock))
    };
    let marker = regular(&output.join(MARKER))?;
    let mut observed = BTreeMap::from([(MARKER.to_owned(), marker.as_deref().map(hash))]);
    let previous = marker.map(|bytes| serde_json::from_slice::<Marker>(&bytes).map_err(|error| format!("Invalid generated API marker: {error}"))).transpose()?;
    if previous.as_ref().is_some_and(|marker| marker.schema_version != 1) { return Err("Unsupported generated API marker version".into()); }
    if previous.is_none() && output.exists() && fs::read_dir(output).map_err(|error| error.to_string())?
        .any(|entry| entry.map_or(true, |entry| entry.file_name() != ".tsrust-api.lock")) {
        return Err("API output must be empty or previously generated; existing application files are preserved".into());
    }
    let next = Marker { schema_version: 1, files: files.iter().map(|(name, contents)| (name.clone(), hash(contents.as_bytes()))).collect() };
    for (name, contents) in &files {
        let current = regular(&output.join(name))?;
        observed.insert(name.clone(), current.as_deref().map(hash));
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
    }
    Ok(OutputPlan { output: output.to_owned(), files, next, observed, _lock: lock })
}

impl OutputPlan {
 fn unchanged(&self) -> Result<(), String> {
    safe_output(&self.output)?;
    for (name, before) in &self.observed {
        if &regular(&self.output.join(name))?.as_deref().map(hash) != before { return Err(format!("Generated API output {name} changed during validation; nothing applied to this output")); }
    }
    Ok(())
 }
 fn apply(&self) -> Result<(), String> {
    // All contracts and owner-edit checks complete before the first replacement.
    // A failed partial write is recoverable: preflight accepts old or planned bytes.
    self.unchanged()?;
    for (name, contents) in &self.files {
        if regular(&self.output.join(name))?.as_deref() != Some(contents.as_bytes()) { atomic(&self.output.join(name), contents.as_bytes())?; }
    }
    atomic(&self.output.join(MARKER), format!("{}\n", serde_json::to_string_pretty(&self.next).map_err(|error| error.to_string())?).as_bytes())
 }
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
    let path = fs::canonicalize(path).map_err(|error| error.to_string())?;
    let root = path.parent().ok_or("API config needs a parent directory")?;
    let inputs = Candidate::new(BTreeMap::new())?;
    let config_source = inputs.read(&SourceId(path.to_string_lossy().into_owned()))?;
    let config: Config = serde_json::from_str(&config_source).map_err(|error| format!("{}: {error}", path.display()))?;
    if config.schema_version != 1 { return Err("API configuration schemaVersion must be 1".into()); }
    let entry = root.join(relative(&config.entry)?);
    let output = output.map(|path| cwd.join(path)).unwrap_or(root.join(relative(&config.output)?));
    safe_output(&output)?;
    let output = normalize(&output)?;
    let mut selections = config.operations;
    for selection in &mut selections {
        let path = root.join(relative(&selection.module)?);
        selection.module = inputs.resolve(None, path.to_str().ok_or("API source path must be UTF-8")?)?.0;
    }
    let graph = load_project(&inputs, entry.to_str().ok_or("API entry must be UTF-8")?).map_err(|error| error.message)?;
    let project = check_graph(graph).map_err(|error| error.message)?;
    let api = CheckedApi::from_project(&project, &selections)?;
    let files = api.manifest.client_package(&config.package.name, &config.package.version)?;
    let mut packages = vec![(output.clone(), files)];
    let mut native_inputs = None;
    if let Some(server) = config.server {
        let directory = root.join(relative(&server.output)?);
        safe_output(&directory)?;
        let directory = normalize(&directory)?;
        if directory.starts_with(&output) || output.starts_with(&directory) { return Err("Private server output and public client output must be separate directories".into()); }
        let resolve = |import: Import| -> Result<NativeImport, String> {
            let path = root.join(relative(&import.module)?);
            Ok(NativeImport { module: inputs.resolve(None, path.to_str().ok_or("API import path must be UTF-8")?)?.0, export: import.export })
        };
        let context = resolve(server.context)?;
        let authorize = resolve(server.authorize)?;
        let mut previous = Vec::new();
        for retained in server.compatible {
            let path = root.join(relative(&retained)?);
            let id = inputs.resolve(None, path.to_str().ok_or("Retained manifest path must be UTF-8")?)?;
            previous.push(serde_json::from_str::<Manifest>(&inputs.read(&id)?).map_err(|error| format!("Invalid retained API manifest: {error}"))?);
        }
        let entry = directory.join("index.tsr");
        let options = NativeServer { context, authorize, previous, max_request_bytes: server.max_request_bytes, max_response_bytes: server.max_response_bytes };
        let source = api.native_source(&entry, &options)?;
        let source = tsrust_compiler::format(&source).map_err(|error| error.message)?;
        if tsrust_compiler::format(&source).map_err(|error| error.message)? != source { return Err("Generated API source formatting is not stable".into()); }
        let id = SourceId(entry.to_string_lossy().into_owned());
        let candidate = Candidate::new(BTreeMap::from([(id, source.clone())]))?;
        let render = |error| tsrust_compiler::project_cli::InputDiagnostic::Project(error).render_with_provider(&entry.to_string_lossy(), &source, false, &candidate);
        let graph = load_project(&candidate, entry.to_str().ok_or("API server path must be UTF-8")?).map_err(&render)?;
        check_graph(graph).map_err(render)?;
        native_inputs = Some(candidate);
        packages.push((directory, BTreeMap::from([("index.tsr".into(), source)])));
    }
    // Acquire all output locks and finish ownership checks before replacing either
    // side. Source snapshots also remain pinned through the complete preflight.
    let plans = packages.into_iter().map(|(output, files)| prepare_package(&output, files, check)).collect::<Result<Vec<_>, _>>()?;
    inputs.unchanged()?;
    if let Some(candidate) = native_inputs { candidate.unchanged()?; }
    for plan in &plans { plan.unchanged()?; }
    if !check { for plan in &plans { plan.apply()?; } }
    println!("{} API client{}: {} operations, contract {}", if check { "Verified" } else { "Generated" }, if plans.len() > 1 { " and private native dispatch" } else { "" }, api.manifest.operations.len(), api.manifest.identity());
    Ok(0)
}

