mod support;
use support::{TempDirectory, run, success};

const OWNER: &str = include_str!("../../../experiments/ui-tsrust-owner-handles/witnesses/owner.tsr");
const NAMED: &str = include_str!("../../../experiments/ui-tsrust-owner-handles/witnesses/named-self-replacement.tsr");
const FACTORY: &str = include_str!("../../../experiments/ui-tsrust-owner-handles/witnesses/factory-self-replacement.tsr");

fn project(source: &str) -> TempDirectory {
    let directory = TempDirectory::new();
    std::fs::write(directory.0.join("owner.tsr"), support::format_round_trip(OWNER)).unwrap();
    std::fs::write(directory.0.join("main.tsr"), support::format_round_trip(source)).unwrap();
    directory
}

#[test]
fn named_handler_discovery_admits_access_summary_before_reorder_call() {
    // Reduced B4: the failure occurs during discovery, without event delivery.
    let source = r#"
import {strictEqual} from 'std:testing';
import {Domain,Owner,Handle,create,root,mount,add,reorder} from './owner.tsr';
function increment(owner:Mutable<Owner>,handle:ReadOnly<Handle>,delta:int):void {add(owner,handle,delta);}
function make(parent:Handle):(owner:Mutable<Owner>,handle:ReadOnly<Handle>,delta:int)=>void {
 return (owner:Mutable<Owner>,handle:ReadOnly<Handle>,delta:int):void=>{
  mount(owner,parent,'new',10,increment);
  strictEqual(reorder(owner,parent,['new','sibling']),true);
 };
}
function main():void {const domain:Domain={next:1};const owner=create(domain);const parent=root(owner);mount(owner,parent,'action',0,make(clone(parent)));mount(owner,parent,'sibling',7,increment);}
"#;
    let directory = project(source);
    for command in ["check", "emit"] {
        success(&run(support::cli().arg(command).arg(directory.0.join("main.tsr")), 30));
    }
}

#[test]
fn named_and_factory_handlers_execute_with_reordered_declarations_and_reuse() {
    for source in [NAMED.to_owned(), FACTORY.to_owned(), {
        let start = NAMED.find("function increment(").unwrap();
        let end = NAMED.find("function replace(").unwrap();
        format!("{}{}\n{}", &NAMED[..start], &NAMED[end..], &NAMED[start..end])
    }] {
        let directory = project(&source);
        let emitted = directory.0.join("program.rs");
        success(&run(support::cli().arg("check").arg(directory.0.join("main.tsr")),30));
        success(&run(support::cli().arg("emit").arg(directory.0.join("main.tsr")).arg("-o").arg(&emitted),30));
        let binary = directory.0.join("program");
        success(&run(support::rustc().arg("--edition=2024").arg(&emitted).arg("-o").arg(&binary),30));
        let output = run(&mut std::process::Command::new(binary),30);
        success(&output);
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "PASS active self-replacement, stale queued event, unaffected FIFO, reentrant rejection, final root removal\n");
    }
}
