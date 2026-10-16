mod support;
fn root()->std::path::PathBuf{std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")}
#[test]
fn service_examples_check_and_signal_loans_reject(){
 for example in ["examples/service/main.tsr","examples/service/contracts.tsr","examples/service-observability/main.tsr"]{support::success(&support::run(support::cli().arg("check").arg(root().join(example)),30));}
 let directory=support::TempDirectory::new();
 for source in ["import {listen,receive} from 'std:signals';async function main():Task<void>{const signal=unwrap(listen());const a=receive(signal);const b=receive(signal);await a;await b;}","import {listen} from 'std:signals';async function main():Task<void>{const signal=unwrap(listen());const copy=clone(signal);}","import {App} from 'std:http';function main(){const app=new App();app.configure({bodyLimit:1});}"]{let output=support::run(support::cli().arg("check").arg(directory.source(&support::format_round_trip(source))),30);assert!(!output.status.success(),"accepted invalid service contract");}
}
// Native contracts, observability and deployment run together in tools/check-service.mjs.
