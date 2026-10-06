import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertABI, adaptRustDriver, adaptTSDriver, rustChecksum, assertCLIInputs} from './adapters.mjs';
const nativeDriver=fs.readFileSync(new URL('./driver.rs',import.meta.url),'utf8');
const probeDriver=fs.readFileSync(new URL('./allocation-driver.rs',import.meta.url),'utf8');
const tsDriver=fs.readFileSync(new URL('./driver.ts',import.meta.url),'utf8');
const row={id:'scalar_query',result:'string[]',inputText:'A😀Z',scalarCoordinates:true};
test('native and Node coordinate preparation remains outside workload timing and allocator counters',()=>{
 const native=adaptRustDriver(nativeDriver,row);
 assert.ok(native.indexOf('input.chars().count()') < native.indexOf('for _ in 0..warmups'));
 assert.equal((native.match(/black_box\(seed\), &input, coordinate_width/g)||[]).length,2);
 const probe=adaptRustDriver(probeDriver,row,{probe:true});
 assert.ok(probe.indexOf('input.chars().count()') < probe.indexOf('let base_live'));
 assert.match(probe,/function0\(std::hint::black_box\(n\), std::hint::black_box\(7\), &input, coordinate_width\)/);
 const node=adaptTSDriver(tsDriver,row);
 assert.ok(node.indexOf('Array.from(argv[4]).length') < node.indexOf('performance.now()'));
 assert.equal((node.match(/workload\(iterations, seed, argv\[4\], coordinateWidth\)/g)||[]).length,2);
});
test('ABI rejects missing coordinate parameters and literal queries require no invented external input',()=>{
 const emitted='fn function0(value0: i64, value1: i64, value2: &String, value3: i64) -> Array<String> {} fn main() {}';
 assert.doesNotThrow(()=>assertABI(emitted,row));
 assert.throws(()=>assertABI(emitted.replace(', value3: i64',''),row),/Emitter ABI changed/);
 const literal={id:'literal',result:'string[]',scalarCoordinates:true,literalScalarCount:32770};
 const native=adaptRustDriver(nativeDriver,literal);
 assert.match(native,/let coordinate_width = 32770_i64;/);
 assert.doesNotMatch(native,/let input/);
 assert.match(native,/black_box\(seed\), coordinate_width,/);
 assert.match(rustChecksum(row),/value.chars\(\)/);
 assert.doesNotMatch(rustChecksum(row),/raw_units|as_str|units\(/);
});

test('CLI input guard rejects NUL before workload preparation without banning literal coverage',()=>{
 assert.throws(()=>assertCLIInputs([{id:'external',inputText:'a\0b'}]),/CLI string inputs cannot contain NUL/);
 assert.doesNotThrow(()=>assertCLIInputs([{id:'literal'}, {id:'empty',inputText:''}, {id:'unicode',inputText:'Straße 😀'}]));
});

test('ordinary native drivers bind the complete workload signature after specialization',()=>{
 const workload={id:'generic',result:'int[][]',inputText:'abc'};
 const entry='fn function5(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {}';
 const helpers='fn function0(value0: i64, value1: &mut Record0) -> i64 {}\n';
 const emitted=helpers+entry+'\nfn main() {}';
 assert.equal(assertABI(emitted,workload).functionName,'function5');
 const driver=adaptRustDriver(nativeDriver,workload,{emitted});
 assert.match(driver,/function5\(\s*std::hint::black_box\(iterations\)/);
 assert.doesNotMatch(driver,/\bfunction0\s*\(/);
 const probe=adaptRustDriver(probeDriver,workload,{probe:true,emitted});
 assert.match(probe,/function5\(std::hint::black_box\(n\)/);
 const original=emitted.replaceAll('function5','function0').replace(helpers,'');
 assert.equal(adaptRustDriver(nativeDriver,workload,{emitted:original}),adaptRustDriver(nativeDriver,workload));
 assert.throws(()=>assertABI(emitted+'\n'+entry.replace('function5','function6'),workload),/expected one complete workload signature/);
 assert.throws(()=>assertABI(helpers+'fn main() {}',workload),/expected one complete workload signature/);
});

test('workload-first source disambiguates same-signature helpers without guessing other declarations',()=>{
 const row={id:'array_temporary_index_string',result:'string[]',inputText:'abc'};
 const signature='(value0: i64, value1: i64, value2: &String) -> Array<String> {}';
 const emitted='fn function0'+signature+'\nfn function2'+signature+'\nfn main() {}';
 const source='function workload(n: int, seed: int, input: string): string[] { return []; } function helper(n: int, seed: int, input: string): string[] { return []; }';
 assert.equal(assertABI(emitted,row,{source}).functionName,'function0');
 assert.throws(()=>assertABI(emitted,row),/expected one complete/);
 assert.throws(()=>assertABI(emitted,row,{source:'function helper(): int { return 0; } '+source}),/expected one complete/);
 assert.throws(()=>assertABI(emitted.replace('function0','function1'),row,{source}),/expected one complete/);
});
