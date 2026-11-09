mod support;
#[test]
fn collection_numeric_date_completion_executes_formatted_fixtures(){
 for source in [include_str!("../../../examples/stdlib-completion/collections.tsr"),include_str!("../../../examples/stdlib-completion/numeric.tsr"),include_str!("../../../examples/stdlib-completion/date.tsr")]{
  let directory=support::TempDirectory::new();let source=support::format_round_trip(source);
  support::success(&support::run(support::cli().arg("run").arg(directory.source(&source)),180));
 }
}
#[test]
fn collection_endpoint_borrows_and_snapshot_cloneability_are_enforced(){
 for source in [
  "import {OrderedSet} from 'std:collections';function main(){const values=new OrderedSet<string>();values.add('a');const key=values.first()!;values.clear();console.log(key);}",
  "import {OrderedSet} from 'std:collections';function main(){const values=new OrderedSet<string>();values.add('a');const key=values.last()!;const moved=values;console.log(key,moved.size);}",
  "import {OrderedMap} from 'std:collections';function main(){const values=new OrderedMap<string,int>();values.set('a',1);const key=values.firstKey()!;values.clear();console.log(key);}",
  "import {OrderedMap} from 'std:collections';function main(){const values=new OrderedMap<string,int>();values.set('a',1);const key=values.lastKey()!;const moved=values;console.log(key,moved.size);}",
  "import {OrderedSet} from 'std:collections';function main(){const values=new OrderedSet<int>();values.add(1);values.forEach(value=>{values.clear();});}",
  "import {Deque} from 'std:collections';type OnceValue={call:Once<Function<int>>;};function main(){const values=new Deque<OnceValue>();values.pushBack({call:()=>1});values.values();}",
  "function main(){const map=new Map<int,int>();map.firstKey();}",
  "import {parseInt as exact} from 'std:numeric';function main(){exact('1',2.5);}",
 ]{
  let directory=support::TempDirectory::new();let source=support::format_round_trip(source);
  let output=support::run(support::cli().arg("check").arg(directory.source(&source)),30);assert!(!output.status.success(),"accepted: {source}");
 }
}

// Runtime source is normally emitted as text; include its focused backend tests
// explicitly without registering another copy of any broader runtime suite.
#[allow(dead_code, reason="Exercise the standalone numeric backend boundary witness.")]
mod numeric_runtime {
    include!("../src/stdlib/numeric/runtime.rs");
}
