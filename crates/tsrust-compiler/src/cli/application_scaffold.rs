//! Application-owned starter source. SDK generation owns only its marked outputs.
use std::{collections::BTreeMap, fs, io::Write, path::Path};
use sha2::{Digest, Sha256};

pub(super) fn validate_targets(targets: &[String]) -> Result<(), String> {
    if targets.len() > 1 || targets.first().is_some_and(|target| !matches!(target.as_str(), "web" | "backend")) {
        return Err("Application targets currently supported: web (default) or backend. Select one; desktop/mobile integration is not implemented yet.".into());
    }
    Ok(())
}

pub(super) fn create(root: &Path, targets: &[String]) -> Result<u8, String> {
    validate_targets(targets)?;
    let web = targets.first().is_none_or(|target| target == "web");
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
    files.insert(".gitignore", "/target/\n**/node_modules/\n**/dist/\n*.tsbuildinfo\n.env.local\n".into());
    if web {
        let workspace = serde_json::json!({"name":name.to_ascii_lowercase(),"private":true,"version":"0.1.0","packageManager":"pnpm@12.3.4","scripts":{"api":"tsrust api generate","dev:backend":"tsrust dev","dev:web":"pnpm --dir web dev","build:web":"pnpm --dir web build"}});
        files.insert("package.json", format!("{}\n",serde_json::to_string_pretty(&workspace).unwrap()));
        files.insert("pnpm-workspace.yaml", "packages:\n  - web\n  - generated/client\nnodeLinker: hoisted\n".into());
        template!("pnpm-lock.yaml");
        template!("web/package.json"); template!("web/tsconfig.json"); template!("web/vite.config.ts");
        template!("web/index.html"); template!("web/components.json"); template!("web/LICENSE-vite.txt"); template!("web/LICENSE-shadcn.txt");
        template!("web/public/Geist-OFL.txt");
        template!("web/public/icon.svg");
        template!("web/src/main.tsx"); template!("web/src/App.tsx"); template!("web/src/index.css"); template!("web/src/lib/api.ts"); template!("web/src/lib/utils.ts");
        template!("web/src/components/ui/button.tsx"); template!("web/src/components/ui/input.tsx"); template!("web/src/components/ui/card.tsx");
        template!("web/src/components/ui/field.tsx"); template!("web/src/components/ui/label.tsx"); template!("web/src/components/ui/separator.tsx");
    } else {
        files.insert("README.md", "# Tsrust backend\n\nThe public greeting API listens on loopback port 4318.\n\nRun `tsrust dev`, `tsrust check`, `tsrust test`, and `tsrust api generate`.\nThe checked client is in generated/client; import or export it into any frontend.\nNo frontend toolchain or dependency install is required for this target.\nRead AGENTS.md and app/api/access.tsr before exposing sensitive operations.\nPut reusable backend logic in app/lib and its tests in app/tests.\n".into());
    }
    // Validate and format every Tsrust template before creating the destination.
    for (path, source) in &mut files {
        if path.ends_with(".tsr") {
            *source = tsrust_compiler::format(source).map_err(|error| format!("Cannot format starter {path}: {}",error.message))?;
        }
    }
    let hashes: BTreeMap<&str, String> = files.iter().map(|(path, source)| (*path, Sha256::digest(source.as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect())).collect();
    let integration = serde_json::json!({"schemaVersion":1,"template":if web {"application-web"} else {"application-backend"},"integrationVersion":"0.2.0","targets":if web {vec!["web"]} else {vec!["backend"]},"initialFiles":hashes});
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
    if web { println!("Web: cd web, pnpm install, then pnpm dev. No dependencies were installed automatically."); }
    Ok(0)
}
