//! Frontend-only module workloads; timing is opt-in at the scheduled checkpoint.
use std::cell::Cell;
use std::collections::BTreeMap;
use std::time::Instant;
use tsrust_compiler::project::{SourceId, SourceProvider, check_graph, load_project};

struct Workload {
    sources: BTreeMap<String, String>,
    reads: Cell<usize>,
}
impl Workload {
    fn fanout(modules: usize) -> Self {
        let mut sources = BTreeMap::new();
        sources.insert(
            "shared.tsr".into(),
            "export function seed():int{return 1;}".into(),
        );
        let mut entry = String::new();
        for module in 0..modules {
            entry.push_str(&format!(
                "import {{ value as value{module} }} from './part{module}.tsr';\n"
            ));
            sources.insert(format!("part{module}.tsr"),format!("import {{ seed }} from './shared.tsr';export function value():int{{return seed()+{module};}}"));
        }
        entry.push_str("function main(){let total=0;");
        for module in 0..modules {
            entry.push_str(&format!("total=total+value{module}();"));
        }
        entry.push_str("console.log(total);}");
        sources.insert("main.tsr".into(), entry);
        Self {
            sources,
            reads: Cell::new(0),
        }
    }
}
impl SourceProvider for Workload {
    fn resolve(&self, _: Option<&SourceId>, path: &str) -> Result<SourceId, String> {
        let id = path.trim_start_matches("./");
        if self.sources.contains_key(id) {
            Ok(SourceId(id.into()))
        } else {
            Err(format!("Missing {id}"))
        }
    }
    fn read(&self, id: &SourceId) -> Result<String, String> {
        let source=self.sources.get(&id.0).ok_or("Missing source")?;
        // Count module reads; an absent optional project configuration is not a module.
        self.reads.set(self.reads.get() + 1);
        Ok(source.clone())
    }
}
#[test]
fn shared_dependency_fanout_reads_once_and_checks() {
    for modules in [16, 64] {
        let workload = Workload::fanout(modules);
        let graph = load_project(&workload, "main.tsr").unwrap();
        assert_eq!(graph.modules.len(), modules + 2);
        assert_eq!(workload.reads.get(), modules + 2);
        let checked = check_graph(graph).unwrap_or_else(|e| panic!("{}: {}", e.code, e.message));
        assert!(checked.emit_rust().contains("fn main"));
    }
}
#[test]
#[ignore = "Run frontend measurements only at the scheduled five-gate checkpoint"]
fn measure_module_frontend_fanout() {
    let samples = std::env::var("TSRUST_MODULE_SAMPLES")
        .ok()
        .map(|s| s.parse::<usize>().expect("sample count"))
        .unwrap_or(10);
    assert!((1..=1000).contains(&samples));
    for modules in [16, 64, 256] {
        let workload = Workload::fanout(modules);
        let bytes = workload.sources.values().map(String::len).sum::<usize>();
        let expected = modules * (modules + 1) / 2;
        for sample in 0..samples {
            workload.reads.set(0);
            let start = Instant::now();
            let graph = load_project(&workload, "main.tsr").unwrap();
            let load = start.elapsed().as_nanos();
            let start = Instant::now();
            let checked = check_graph(graph).unwrap_or_else(|e| panic!("{}", e.message));
            let check = start.elapsed().as_nanos();
            let start = Instant::now();
            let rust = checked.emit_rust();
            let emit = start.elapsed().as_nanos();
            println!(
                "{{\"workload\":\"module-shared-fanout\",\"module_count\":{},\"source_bytes\":{bytes},\"sample\":{sample},\"load_ns\":{load},\"check_ns\":{check},\"emit_ns\":{emit},\"generated_bytes\":{},\"provider_reads\":{},\"expected_native_total\":{expected}}}",
                modules + 2,
                rust.len(),
                workload.reads.get()
            );
        }
    }
}
