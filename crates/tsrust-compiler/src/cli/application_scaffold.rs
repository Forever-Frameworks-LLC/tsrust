//! Application-owned starter source. SDK generation owns only its marked outputs.
use std::{collections::BTreeMap, fs, io::Write, path::Path};
use sha2::{Digest, Sha256};

pub(super) fn validate_targets(targets: &[String]) -> Result<(), String> {
    if targets.len() > 1 || targets.first().is_some_and(|target| target != "backend") {
        return Err("Applications are backend-only. Omit --target or select backend; bring your own UI and call the HTTP API.".into());
    }
    Ok(())
}

pub(super) fn create(root: &Path, targets: &[String]) -> Result<u8, String> {
    validate_targets(targets)?;
    let name = root.file_name().and_then(|value| value.to_str()).ok_or("Project name must be UTF-8")?;
    let mut files: BTreeMap<&str, String> = BTreeMap::new();
    // Raw assets use .template so editor services do not treat pre-generation
    // imports/configs as runnable projects. Generated files keep normal extensions.
    macro_rules! template { ($path:literal) => { files.insert($path, include_str!(concat!("../../templates/application/", $path, ".template")).into()); }; }
    template!("app/main.tsr");
    template!("app/api/context.tsr");
    template!("app/api/access.tsr");
    template!("app/api/operations.tsr");
    template!("app/lib/greeting.tsr");
    template!("app/tests/greet.test.tsr");
    template!("AGENTS.md");
    template!("README.md");
    let api = serde_json::json!({"entry":"app/api/operations.tsr","output":"generated/client",
        "package":{"name":"@app/api","version":"0.1.0"},
        "server":{"output":"generated/server","context":{"module":"app/api/context.tsr","export":"AppContext"},"authorize":{"module":"app/api/access.tsr","export":"authorize"}},
        "operations":[{"name":"greet","module":"app/api/operations.tsr","export":"greet","kind":"mutation"}]});
    let config = serde_json::json!({"schemaVersion":1,"name":name,"build":{"entry":"app/main.tsr","output":format!("target/{name}{}",std::env::consts::EXE_SUFFIX)},"api":api});
    files.insert(super::project_config::FILE, format!("{}\n", serde_json::to_string_pretty(&config).unwrap()));
    files.insert(".gitignore", "/target/\n.env.local\n".into());
    // Validate and format every Tsrust template before creating the destination.
    for (path, source) in &mut files {
        if path.ends_with(".tsr") {
            *source = tsrust_compiler::format(source).map_err(|error| format!("Cannot format starter {path}: {}",error.message))?;
        }
    }
    let hashes: BTreeMap<&str, String> = files.iter().map(|(path, source)| (*path, Sha256::digest(source.as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect())).collect();
    let integration = serde_json::json!({"schemaVersion":1,"template":"application-backend","integrationVersion":"0.2.0","targets":["backend"],"initialFiles":hashes});
    files.insert("generated/.tsrust-integration.json", format!("{}\n",serde_json::to_string_pretty(&integration).unwrap()));
    // Exclusive creation prevents overwriting files, directories and symlinks.
    fs::create_dir(root).map_err(|error| format!("Cannot create {}: {error}. Existing projects are never overwritten.",root.display()))?;
    let write = || -> Result<(), String> {
        for (path, source) in files {
            let destination = root.join(path);
            fs::create_dir_all(destination.parent().unwrap()).map_err(|error| error.to_string())?;
            let mut file = fs::OpenOptions::new().write(true).create_new(true).open(destination).map_err(|error| error.to_string())?;
            file.write_all(source.as_bytes()).map_err(|error| error.to_string())?;
        }
        let config = fs::canonicalize(root.join(super::project_config::FILE)).map_err(|error| error.to_string())?;
        super::api_cli::run(["generate".into(), "--config".into(), config.to_string_lossy().into_owned()].into_iter())?;
        Ok(())
    };
    write().map_err(|error| format!("Application creation failed: {error}. Partial files remain in {}; inspect them before retrying with a new destination.",root.display()))?;
    println!("Created {} with a checked public greeting API.\nBackend: cd {} then tsrust dev",root.display(),root.display());
    Ok(0)
}
