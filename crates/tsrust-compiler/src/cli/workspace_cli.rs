//! Workspaces: one root `tsrust.workspace.json` beside independent member projects.
//! At the root, check/build/test/format run each member after its client edges are
//! generated (or checked for staleness). Inside a member nothing changes.
use serde::Deserialize;
use std::{collections::BTreeMap, env, fs, path::{Component, Path, PathBuf}, process::Command};

pub const FILE: &str = "tsrust.workspace.json";

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Manifest {
    schema_version: u32,
    projects: BTreeMap<String, String>,
    #[serde(default)]
    clients: Vec<Client>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Client {
    project: String,
    from: String,
    language: String,
    output: String,
}

struct Workspace {
    projects: BTreeMap<String, PathBuf>,
    clients: Vec<Client>,
}

fn inside(value: &str, at: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);
    if value.is_empty() || value.contains('\\') || path.components().any(|part| !matches!(part, Component::Normal(_))) {
        return Err(format!("{FILE}: {at} must be a relative path inside the workspace, using '/'"));
    }
    Ok(path.to_owned())
}

fn parse(root: &Path, text: &str) -> Result<Workspace, String> {
    let manifest: Manifest = serde_json::from_str(text).map_err(|error| format!("{FILE}: {error}"))?;
    if manifest.schema_version != 1 { return Err(format!("{FILE}: schemaVersion must be 1")); }
    if manifest.projects.is_empty() { return Err(format!("{FILE}: projects must name at least one member")); }
    let mut projects = BTreeMap::new();
    for (name, directory) in &manifest.projects {
        if !super::project_config::valid_name(name) { return Err(format!("{FILE}: project name {name:?} must start with a letter and contain only ASCII letters, digits, '-' or '_'")); }
        let relative = inside(directory, &format!("projects.{name}"))?;
        if projects.values().any(|other: &PathBuf| relative.starts_with(other) || other.starts_with(&relative)) {
            return Err(format!("{FILE}: project {name:?} overlaps another member"));
        }
        let member = root.join(&relative);
        if !member.join(super::project_config::FILE).is_file() { return Err(format!("{FILE}: project {name:?} at {directory} has no {}", super::project_config::FILE)); }
        projects.insert(name.clone(), relative);
    }
    for client in &manifest.clients {
        for name in [&client.project, &client.from] {
            if !projects.contains_key(name) { return Err(format!("{FILE}: client refers to unknown project {name:?}")); }
        }
        if client.project == client.from { return Err(format!("{FILE}: project {:?} cannot generate a client from itself", client.project)); }
        if !matches!(client.language.as_str(), "typescript" | "rust") { return Err(format!("{FILE}: client language must be typescript or rust")); }
        inside(&client.output, "clients.output")?;
    }
    Ok(Workspace { projects: projects.into_iter().map(|(name, relative)| (name, root.join(relative))).collect(), clients: manifest.clients })
}

/// Contract providers run before the members that consume their clients.
fn order(workspace: &Workspace, selected: Option<&str>) -> Vec<String> {
    let mut names: Vec<String> = workspace.projects.keys().filter(|name| selected.is_none_or(|selected| selected == name.as_str())).cloned().collect();
    names.sort_by_key(|name| workspace.clients.iter().any(|client| &client.project == name));
    names
}

fn status(command: &mut Command) -> Result<u8, String> {
    let status = command.status().map_err(|error| error.to_string())?;
    Ok(if status.success() { 0 } else { u8::try_from(status.code().unwrap_or(1)).unwrap_or(1).max(1) })
}

/// Handle a root-level command, or return None when this is not a workspace invocation.
pub fn dispatch(command: &str, args: &[String]) -> Option<Result<u8, String>> {
    if !matches!(command, "check" | "build" | "test" | "format") { return None; }
    let root = env::current_dir().ok()?;
    let text = fs::read_to_string(root.join(FILE)).ok()?;
    let mut rest = Vec::new();
    let mut selected = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--project" {
            let Some(name) = iter.next() else { return Some(Err("--project requires a project name".into())); };
            selected = Some(name.clone());
        } else if !arg.starts_with('-') && selected.is_none() && rest.iter().all(|value: &String| value.starts_with('-')) {
            // An explicit input path is an ordinary single-project invocation.
            return None;
        } else {
            rest.push(arg.clone());
        }
    }
    Some((|| {
        let workspace = parse(&root, &text)?;
        if let Some(name) = &selected && !workspace.projects.contains_key(name) { return Err(format!("{FILE}: unknown project {name:?}")); }
        let executable = env::current_exe().map_err(|error| error.to_string())?;
        let mut code = 0;
        if command != "format" {
            for client in workspace.clients.iter().filter(|client| selected.as_ref().is_none_or(|name| name == &client.project)) {
                let provider = &workspace.projects[&client.from];
                let config = provider.join(super::project_config::FILE);
                let output = workspace.projects[&client.project].join(&client.output);
                let mut generate = Command::new(&executable);
                generate.args(["api", "generate"]);
                // Explicit `api` settings describe the contract; otherwise the
                // provider's router entry does, as for `api generate --entry`.
                let inline = fs::read_to_string(&config).ok().and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok()).is_some_and(|value| value.get("api").is_some());
                if inline { generate.arg("--config").arg(&config); } else { generate.arg("--entry").arg(super::project_config::load(provider)?.entry); }
                generate.args(["--language", &client.language, "--out"]).arg(&output);
                // `check` never rewrites sources: a stale client is an error.
                if command == "check" { generate.arg("--check"); }
                code = code.max(status(&mut generate)?);
            }
        }
        for name in order(&workspace, selected.as_deref()) {
            println!("{name}: tsrust {command}");
            code = code.max(status(Command::new(&executable).arg(command).args(&rest).current_dir(&workspace.projects[&name]))?);
        }
        Ok(code)
    })())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn workspace(members: &[&str]) -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        for member in members { fs::create_dir_all(root.path().join(member)).unwrap(); fs::write(root.path().join(member).join("tsrust.config.json"), "{}").unwrap(); }
        root
    }
    #[test]
    fn manifests_validate_members_paths_and_client_edges() {
        let root = workspace(&["backend", "frontend"]);
        let valid = r#"{"schemaVersion":1,"projects":{"api":"backend","app":"frontend"},"clients":[{"project":"app","from":"api","language":"typescript","output":"src/generated/api"}]}"#;
        let parsed = parse(root.path(), valid).unwrap();
        assert_eq!(order(&parsed, None), ["api", "app"]);
        assert_eq!(order(&parsed, Some("app")), ["app"]);
        for (text, message) in [
            (r#"{"schemaVersion":2,"projects":{"api":"backend"}}"#, "schemaVersion"),
            (r#"{"schemaVersion":1,"projects":{}}"#, "at least one"),
            (r#"{"schemaVersion":1,"projects":{"api":"../backend"}}"#, "inside the workspace"),
            (r#"{"schemaVersion":1,"projects":{"api":"backend","nested":"backend/inner"}}"#, "overlaps"),
            (r#"{"schemaVersion":1,"projects":{"api":"missing"}}"#, "has no"),
            (r#"{"schemaVersion":1,"projects":{"api":"backend"},"clients":[{"project":"app","from":"api","language":"rust","output":"x"}]}"#, "unknown project"),
            (r#"{"schemaVersion":1,"projects":{"api":"backend","app":"frontend"},"clients":[{"project":"app","from":"api","language":"go","output":"x"}]}"#, "language"),
            (r#"{"schemaVersion":1,"projects":{"api":"backend","app":"frontend"},"clients":[{"project":"app","from":"api","language":"rust","output":"/abs"}]}"#, "inside the workspace"),
            (r#"{"schemaVersion":1,"projects":{"api":"backend"},"extra":true}"#, "unknown field"),
        ] {
            let error = parse(root.path(), text).err().unwrap_or_else(|| panic!("accepted {text}"));
            assert!(error.contains(message), "{text}: {error}");
        }
    }
}
