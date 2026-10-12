//! Recursive compiler diagnostics; this path never emits or executes native code.
use std::{collections::BTreeMap, env, fs, io::IsTerminal, path::PathBuf};
use tsrust_compiler::{Diagnostic, Span, json_string, project_cli::{InputDiagnostic, check_input_diagnostics}};

pub(super) fn run(mut args: impl Iterator<Item=String>) -> Result<u8,String> {
    let mut inputs=Vec::new();let mut json=false;let mut color="auto".to_owned();
    while let Some(arg)=args.next() {
        match arg.as_str() {
            "--json" if !json => json=true,
            "--release" => return Err("--release applies only to build/run/test/dev".into()),
            "--color" => color=args.next().ok_or("--color requires auto, always or never")?,
            _ if arg.starts_with("--color=") => color=arg[8..].to_owned(),
            _ if arg.starts_with('-') => return Err(format!("Unknown or repeated check option: {arg}")),
            _ => inputs.push(PathBuf::from(arg)),
        }
    }
    if !matches!(color.as_str(),"auto"|"always"|"never") {return Err("--color requires auto, always or never".into());}
    let single=inputs.len()==1 && inputs[0].is_file();
    if inputs.is_empty(){inputs.push(super::project_config::default_root()?);}
    let files=tsrust_compiler::source_discovery::discover(&inputs)?;
    let color=color=="always" || (color=="auto" && std::io::stderr().is_terminal() && env::var_os("NO_COLOR").is_none() && env::var("TERM").as_deref()!=Ok("dumb"));
    let mut diagnostics=BTreeMap::new();let mut single_result=None;
    for path in files {
        let (source,errors)=match fs::read_to_string(&path) {
            Ok(source)=>{let error=match check_input_diagnostics(&path,&source){Ok(analysis)=>{if single{single_result=Some(analysis);}Vec::new()},Err(errors)=>errors};(source,error)},
            Err(error)=>(String::new(),vec![InputDiagnostic::Single(Diagnostic::new("source-read",error.to_string(),Span::default()))]),
        };
        for error in errors {
        let (file,span,code)=match &error {
            InputDiagnostic::Single(error)=>(path.to_string_lossy().into_owned(),error.span,error.code),
            InputDiagnostic::Project(error)=>(error.location.source.0.clone(),error.location.span,error.code),
        };
        let encoded=error.to_json(&source);
        let encoded=if single { encoded } else { format!("{{\"file\":{},{}",json_string(&file),&encoded[1..]) };
        let rendered=error.render(&path.to_string_lossy(),&source,color);
        diagnostics.entry((file,span.start,span.end,code)).or_insert((encoded,rendered));
        }
    }
    let failed=!diagnostics.is_empty();
    if json {
        if !failed && let Some(result)=single_result {println!("{}",result.to_json());}
        else {println!("{{\"ok\":{},\"diagnostics\":[{}]}}",!failed,diagnostics.values().map(|(encoded,_)|encoded.as_str()).collect::<Vec<_>>().join(","));}
    }else if failed {for (_,rendered) in diagnostics.values(){eprint!("{rendered}");}}
    else if let Some(analysis)=single_result { print_analysis(&analysis); }
    else { println!("Check passed."); }
    Ok(u8::from(failed))
}

fn print_analysis(analysis: &tsrust_compiler::project_cli::CheckedInput) {
            println!("{}", super::styled("Check passed.", "1;32", super::color_enabled(std::io::stdout().is_terminal())));
            for (name, mode) in analysis.contracts() {
                println!("  {name}: {}", mode.as_str());
            }
            for closure in analysis.closures() {
                println!(
                    "  closure {} in {} (depth {}): {}, {}{}",
                    closure.id,
                    closure.function,
                    closure.depth,
                    closure.invocation.as_str(),
                    if closure.escapes {
                        "owned escape"
                    } else {
                        "local"
                    },
                    if closure.template { ", template" } else { "" }
                );
                for param in closure.parameters {
                    println!("    parameter {}: {}", param.name, param.access.as_str());
                }
                for capture in closure.captures {
                    println!(
                        "    {}: storage {}, access {}",
                        capture.name,
                        capture.storage.as_str(),
                        capture.access.as_str()
                    );
                }
            }
}
