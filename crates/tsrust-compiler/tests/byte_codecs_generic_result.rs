mod support;
#[test]
fn generic_result_error_observer_accepts_the_concrete_native_error() {
    let source = "import {ByteError,fromHex} from 'std:bytes';function code<T>(result:Result<T,ByteError>):string{match(result){Ok(_)=>{return 'unexpected-success';},Err(error)=>{return error.code;}}}function main(){console.log(code(fromHex('0')));}";
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(source));
    let graph = tsrust_compiler::project::load_project(&tsrust_compiler::project::FileSourceProvider, path.to_str().unwrap()).unwrap();
    tsrust_compiler::project::check_graph(graph).unwrap();
}
