//! Opt-in cost observation for project analysis and requests. Not a correctness gate.
use serde_json::json;
use std::time::Instant;
use tsrust_lsp::{
    project_diagnostics::source_id, project_intelligence::ProjectIndex,
    project_session::ProjectSession, workspace::Workspace,
};

fn median(mut samples: Vec<f64>) -> f64 {
    samples.sort_by(f64::total_cmp);
    samples[samples.len() / 2]
}
fn timed<T>(repeats: usize, mut run: impl FnMut() -> T) -> f64 {
    median(
        (0..repeats)
            .map(|_| {
                let start = Instant::now();
                std::hint::black_box(run());
                start.elapsed().as_secs_f64() * 1000.0
            })
            .collect(),
    )
}

#[test]
#[ignore = "Opt-in editor cost observation; run alone: cargo test -p tsrust-lsp --test project_scaling -- --ignored --nocapture"]
fn project_analysis_and_request_cost_by_module_count() {
    let directory = std::env::temp_dir().join(format!("tsrust-lsp-scaling-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let directory = directory.canonicalize().unwrap();
    println!(
        "modules,check_ms,checked_index_ms,header_index_ms,hover_ms,completion_ms,references_ms,current_ms"
    );
    for modules in [10usize, 50, 200] {
        // Chains of 64 modules stay under the import depth limit; the entry imports every chain.
        for index in 0..modules {
            let previous = if index % 64 == 0 {
                String::new()
            } else {
                format!(
                    "import {{ step{p}, Row{p} }} from './m{p}.tsr';\n",
                    p = index - 1
                )
            };
            let body = if index % 64 == 0 {
                "return value + 1;".to_owned()
            } else {
                format!(
                    "const row: Row{p} = {{ value: value }};\n    return step{p}(row.value) + 1;",
                    p = index - 1
                )
            };
            std::fs::write(
                directory.join(format!("m{index}.tsr")),
                format!("{previous}/** Step {index}. */\nexport function step{index}(value: int): int {{\n    {body}\n}}\nexport type Row{index} = {{ value: int; }};\n"),
            )
            .unwrap();
        }
        let last = modules - 1;
        let heads = (0..modules)
            .filter(|index| index % 64 == 63 && *index != last)
            .map(|index| format!("import {{ step{index} }} from './m{index}.tsr';\n"))
            .collect::<String>();
        let calls = (0..modules)
            .filter(|index| index % 64 == 63 && *index != last)
            .map(|index| format!("    console.log(step{index}(1));\n"))
            .collect::<String>();
        let entry = format!(
            "{heads}import {{ step{last}, Row{last} }} from './m{last}.tsr';\nfunction main() {{\n{calls}    const row: Row{last} = {{ value: 1 }};\n    console.log(step{last}(row.value));\n}}\n"
        );
        let uri = format!("file://{}/main.tsr", directory.display());
        let id = source_id(&uri).unwrap();
        let mut workspace = Workspace::default();
        workspace.open(id.clone(), 1, entry.clone());
        let check = timed(5, || workspace.snapshot().check(&id).is_ok());
        let input = workspace.snapshot();
        let analysis = input.check(&id).unwrap();
        let checked_index = timed(5, || ProjectIndex::checked(&analysis));
        let header_index = timed(5, || {
            let input = workspace.snapshot();
            ProjectIndex::new(
                &tsrust_compiler::project::load_project_headers(&input, &id.0).unwrap(),
            )
        });
        let index = ProjectIndex::checked(&analysis);
        let call = entry.find(&format!("step{last}(row")).unwrap();
        let line = entry[..call].matches('\n').count();
        let character = call - entry[..call].rfind('\n').unwrap() - 1;
        let params = json!({"position":{"line":line,"character":character},"context":{"includeDeclaration":true}});
        let request = |method: &str| {
            timed(21, || {
                index.request(method, &params, &uri).unwrap().unwrap()
            })
        };
        let hover = request("textDocument/hover");
        let completion = request("textDocument/completion");
        let references = request("textDocument/references");
        let mut session = ProjectSession::default();
        session.workspace.open(id.clone(), 1, entry.clone());
        let input = session.workspace.snapshot();
        input.check(&id).unwrap();
        // Freshness reads the metadata of every closed file the snapshot loaded.
        let current = timed(21, || session.workspace.current(&input));
        println!(
            "{modules},{check:.3},{checked_index:.3},{header_index:.3},{hover:.4},{completion:.4},{references:.4},{current:.4}"
        );
    }
    std::fs::remove_dir_all(&directory).unwrap();
}
