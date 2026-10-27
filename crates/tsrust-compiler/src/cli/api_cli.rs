//! Export plain checked contracts without adopting or rewriting frontend tooling.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::{Component, Path, PathBuf}};
use tsrust_compiler::{api::{ApiHistory, CheckedApi, Manifest, NativeImport, NativeServer, Selection}, project::{SourceId, SourceProvider, load_project, check_graph}};
use super::package_validation::{Candidate, normalize};

const CONFIG: &str = "tsrust.api.json";
const MARKER: &str = ".tsrust-api.json";

fn inline_config(value: &serde_json::Value) -> Result<Config, String> {
    let mut object = value.as_object().cloned().ok_or("api must be an object")?;
    if object.get("schemaVersion").is_some_and(|version| version.as_u64() != Some(1)) {
        return Err("api.schemaVersion must be 1 when provided".into());
    }
    object.insert("schemaVersion".into(), 1.into());
    serde_json::from_value(serde_json::Value::Object(object)).map_err(|error| format!("Invalid api configuration: {error}"))
}

pub(super) fn validate_inline(value: &serde_json::Value) -> Result<(), String> {
    let config = inline_config(value)?;
    relative(&config.entry)?; relative(&config.output)?;
    for operation in &config.operations { relative(&operation.module)?; }
    if let Some(server) = &config.server {
        relative(&server.output)?; relative(&server.context.module)?; relative(&server.authorize.module)?;
    }
    Ok(())
}

fn discover_config(cwd: &Path) -> Result<PathBuf, String> {
    for root in cwd.ancestors() {
        let standalone = root.join(CONFIG);
        let project = root.join(super::project_config::FILE);
        let inline = if project.is_file() {
            let text = fs::read_to_string(&project).map_err(|error| error.to_string())?;
            let value: serde_json::Value = serde_json::from_str(&text).map_err(|error| format!("{}: {error}",project.display()))?;
            value.get("api").is_some()
        } else { false };
        if inline && standalone.is_file() {
            return Err("Both tsrust.config.json api settings and tsrust.api.json exist; select one explicitly with --config".into());
        }
        if inline { return Ok(project); }
        if standalone.is_file() { return Ok(standalone); }
    }
    Err("No API configuration found; add api settings to tsrust.config.json or pass --config for a standalone API configuration".into())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Config { schema_version: u32, entry: String, output: String, package: Package, operations: Vec<Selection>, #[serde(default = "api_version")] api_version: u32, #[serde(default)] compatible: Vec<String>, #[serde(default)] server: Option<Server> }
fn api_version() -> u32 { 1 }
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

fn owned_history(output: &Path, inputs: &Candidate, name: &str) -> Result<ApiHistory, String> {
    safe_output(output)?;
    let read = |file: &str| -> Result<Option<String>, String> {
        let path = output.join(file);
        let Some(bytes) = regular(&path)? else { return Ok(None); };
        let id = inputs.resolve(None, path.to_str().ok_or("API history paths must be UTF-8")?)?;
        let text = inputs.read(&id)?;
        if text.as_bytes() != bytes { return Err("API history changed while reading".into()); }
        Ok(Some(text))
    };
    let Some(marker) = read(MARKER)? else { return Ok(ApiHistory::new(name)); };
    let marker: Marker = serde_json::from_str(&marker).map_err(|error| format!("Invalid API ownership marker: {error}"))?;
    if marker.schema_version != 1 { return Err("Unsupported API ownership marker".into()); }
    let checked = |file: &str| -> Result<Option<String>, String> {
        let text = read(file)?;
        if let Some(text) = &text {
            if marker.files.get(file) != Some(&hash(text.as_bytes())) { return Err(format!("Generated API file {file} has user edits; preserve them before reading compatibility history")); }
        } else if marker.files.contains_key(file) { return Err(format!("Generated API history file {file} is missing")); }
        Ok(text)
    };
    if let Some(text) = checked("compatibility.json")? {
        let history: ApiHistory = serde_json::from_str(&text).map_err(|error| format!("Invalid API compatibility history: {error}"))?;
        history.validate()?;
        if history.package_name != name { return Err("API history belongs to another package; use a separate output".into()); }
        return Ok(history);
    }
    let mut history = ApiHistory::new(name);
    if let Some(text) = checked("manifest.json")? {
        let manifest: Manifest = serde_json::from_str(&text).map_err(|error| format!("Invalid previous API manifest: {error}"))?;
        let package = checked("package.json")?.ok_or("Previous API output needs its package metadata")?;
        let package: serde_json::Value = serde_json::from_str(&package).map_err(|error| error.to_string())?;
        if package["name"].as_str() != Some(name) { return Err("Previous API output belongs to another package".into()); }
        let version = package["version"].as_str().ok_or("Previous SDK version is missing")?;
        history = history.advance(name, version, 1, &manifest)?;
    }
    Ok(history)
}

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
    for name in files.keys().chain(previous.iter().flat_map(|marker| marker.files.keys())) {
        relative(name)?;
        if matches!(name.as_str(), MARKER | ".tsrust-api.lock") { return Err("Generated file list cannot contain internal ownership/lock files".into()); }
        safe_output(output.join(name).parent().ok_or("Generated file needs a parent")?)?;
    }
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
        if name != MARKER { relative(name)?; }
        safe_output(self.output.join(name).parent().ok_or("Generated file needs a parent")?)?;
        if &regular(&self.output.join(name))?.as_deref().map(hash) != before { return Err(format!("Generated API output {name} changed during validation; nothing applied to this output")); }
    }
    Ok(())
 }
 fn apply(&self) -> Result<(), String> {
    // All contracts and owner-edit checks complete before the first replacement.
    // A failed partial write is recoverable: preflight accepts old or planned bytes.
    self.unchanged()?;
    for (name, contents) in &self.files {
        let destination = self.output.join(name);
        let parent = destination.parent().ok_or("Generated file needs a parent")?;
        safe_output(parent)?;
        fs::create_dir_all(parent).map_err(|error| format!("Cannot create {}: {error}", parent.display()))?;
        if regular(&destination)?.as_deref() != Some(contents.as_bytes()) { atomic(&destination, contents.as_bytes())?; }
    }
    atomic(&self.output.join(MARKER), format!("{}\n", serde_json::to_string_pretty(&self.next).map_err(|error| error.to_string())?).as_bytes())
 }
}

pub(super) fn run(args: impl Iterator<Item = String>) -> Result<u8, String> {
    let mut args = args;
    if args.next().as_deref() != Some("generate") { return Err("Usage: tsrust api generate [--config file | --description file | --url HTTPS_URL] [--language typescript|rust] [--out directory] [--header-env Header=ENV] [--check]".into()); }
    let mut config = None;
    let mut description = None;
    let mut url = None;
    let mut headers = Vec::new();
    let mut language = None;
    let mut output = None;
    let mut check = false;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--config" if config.is_none() => config = Some(PathBuf::from(args.next().ok_or("--config requires a file")?)),
            "--description" if description.is_none() => description = Some(PathBuf::from(args.next().ok_or("--description requires a saved HTTP API description")?)),
            "--url" if url.is_none() => url = Some(args.next().ok_or("--url requires an HTTP description URL")?),
            "--header-env" => {
                let value = args.next().ok_or("--header-env requires Header-Name=ENVIRONMENT_VARIABLE")?;
                let (name, variable) = value.split_once('=').filter(|(name, variable)| !name.is_empty() && !variable.is_empty()).ok_or("--header-env requires Header-Name=ENVIRONMENT_VARIABLE")?;
                headers.push((name.to_owned(), variable.to_owned()));
            }
            "--language" if language.is_none() => language = Some(args.next().ok_or("--language requires typescript or rust")?),
            "--out" if output.is_none() => output = Some(PathBuf::from(args.next().ok_or("--out requires a directory")?)),
            "--check" if !check => check = true,
            _ => return Err(format!("Unknown or repeated API generation option: {argument}")),
        }
    }
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    if language.as_deref().is_some_and(|value| !matches!(value, "typescript" | "rust")) { return Err("--language requires typescript or rust".into()); }
    if url.is_some() && (description.is_some() || config.is_some()) { return Err("--url, --description and --config are mutually exclusive".into()); }
    if !headers.is_empty() && url.is_none() { return Err("--header-env requires --url".into()); }
    if description.is_some() || url.is_some() {
        if config.is_some() { return Err("--description and --config are mutually exclusive".into()); }
        let output = output.ok_or("Description generation requires an explicit --out directory")?;
        let inputs = Candidate::new(BTreeMap::new())?;
        let text = if let Some(url) = url {
            // The terminal waits for completion; socket/body I/O stays async.
            let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|_| "Cannot initialize HTTP runtime")?;
            runtime.block_on(super::http_client::fetch_description(&url, &headers))?
        } else {
            let description = cwd.join(description.unwrap());
            if fs::metadata(&description).map_err(|error| error.to_string())?.len() > 1_048_576 { return Err("HTTP API description exceeds 1 MiB".into()); }
            let id = inputs.resolve(None, description.to_str().ok_or("Description path must be UTF-8")?)?;
            inputs.read(&id)?
        };
        let value = tsrust_compiler::api::HttpDescription::parse(&text)?;
        let output = normalize(&cwd.join(output))?;
        let mut retained = value.history.clone();
        retained.merge(&owned_history(&output, &inputs, &value.history.package_name)?)?;
        if retained != value.history {
            return Err("HTTP API description omits retained local SDK history; use a separate output directory or a complete description".into());
        }
        let rust = language.as_deref() == Some("rust");
        let files = if rust { value.rust_package()? } else { value.typescript_package()? };
        let plan = prepare_package(&output, files, check)?;
        inputs.unchanged()?;
        plan.unchanged()?;
        if !check { plan.apply()?; }
        println!("{} {} SDK from HTTP API description", if check { "Verified" } else { "Generated" }, if rust { "Rust" } else { "TypeScript" });
        return Ok(0);
    }
    if language.as_deref() == Some("rust") { return Err("Rust SDK generation requires --description or --url; generate the checked service description first".into()); }
    let path = if let Some(config) = config { cwd.join(config) } else { discover_config(&cwd)? };
    let path = fs::canonicalize(path).map_err(|error| error.to_string())?;
    let root = path.parent().ok_or("API config needs a parent directory")?;
    let inputs = Candidate::new(BTreeMap::new())?;
    let config_source = inputs.read(&SourceId(path.to_string_lossy().into_owned()))?;
    let value: serde_json::Value = serde_json::from_str(&config_source).map_err(|error| format!("{}: {error}", path.display()))?;
    let config: Config = if path.file_name().is_some_and(|name| name == super::project_config::FILE) {
        super::project_config::parse(root,&config_source)?;
        inline_config(value.get("api").ok_or("No api settings in tsrust.config.json")?)?
    } else { serde_json::from_value(value).map_err(|error| format!("{}: {error}",path.display()))? };
    if config.schema_version != 1 { return Err("API configuration schemaVersion must be 1".into()); }
    let entry = root.join(relative(&config.entry)?);
    let canonical_output = normalize(&root.join(relative(&config.output)?))?;
    let output = output.map(|path| cwd.join(path)).unwrap_or(canonical_output.clone());
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
    let mut history = owned_history(&canonical_output, &inputs, &config.package.name)?;
    if output != canonical_output { history.merge(&owned_history(&output, &inputs, &config.package.name)?)?; }
    let mut baselines = config.compatible;
    if let Some(server) = &config.server { baselines.extend(server.compatible.iter().cloned()); }
    for retained in baselines {
        let path = root.join(relative(&retained)?);
        let id = inputs.resolve(None, path.to_str().ok_or("Retained contract path must be UTF-8")?)?;
        let text = inputs.read(&id)?;
        let value: serde_json::Value = serde_json::from_str(&text).map_err(|error| format!("Invalid retained API baseline: {error}"))?;
        if value.get("releases").is_some() {
            let retained: ApiHistory = serde_json::from_value(value).map_err(|error| format!("Invalid retained API history: {error}"))?;
            retained.validate()?;
            for release in retained.releases { history.retain(release.api_version, release.manifest)?; }
        } else {
            history.retain(1, serde_json::from_value::<Manifest>(value).map_err(|error| format!("Invalid retained API manifest: {error}"))?)?;
        }
    }
    history = (if config.server.is_some() { history.advance(&config.package.name, &config.package.version, config.api_version, &api.manifest) }
        else { history.advance_client(&config.package.name, &config.package.version, config.api_version, &api.manifest) })
        .map_err(|error| if check { format!("Generated API is stale: {error}") } else { error })?;
    let files = api.manifest.client_package_versioned(&config.package.name, &config.package.version, &history)?;
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
        let entry = directory.join("index.tsr");
        let options = NativeServer { context, authorize, previous: Vec::new(), max_request_bytes: server.max_request_bytes, max_response_bytes: server.max_response_bytes };
        let source = api.native_source_versioned(&entry, &options, &history)?;
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

