import {isCopying,validateCopyingObservation} from './copying-array-observation.mjs';
import {isRange,validateRangeObservation} from './range-mutation-observation.mjs';
import {isHigher,validateHigherObservation} from './higher-order-observation.mjs';
import {isDeclared,validateDeclaredObservation} from './declared-value-observation.mjs';
import {requiresExactResult,validateExactObservation} from './result-observation.mjs';
import {createRequire} from 'node:module';
import {isReductionSchema,observeReduction} from './reduction-observation.mjs';
import {isApproximate,loadApproximateReference,approximatePointProbeSources} from './approximate-harness.mjs';
import {validateApproximatePointProbe,validateApproximateMeasurement} from './approximate-scalar.mjs';
import {createHash} from 'node:crypto';
import {adaptTSDriver} from './adapters.mjs';
import {isConcat,validateConcatObservation} from './concat-observation.mjs';
import {isSliceResult} from './result-observation.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedExactResult} from './oracle.mjs';
import {isNumberResult,observeNumber,validateNumberObservation} from './number-observation.mjs';
import {expectedNumberResult} from './oracle.mjs';
import fs from 'node:fs';
import vm from 'node:vm';
import test from 'node:test';
import assert from 'node:assert/strict';
import ts from './node_modules/typescript/lib/typescript.js';
import {expectedChecksum, unicodeScalars, byteBoundaries, literalMiddleScalars, expectedStringSearchResult} from './oracle.mjs';
const manifest = JSON.parse(fs.readFileSync(new URL('./manifest.json', import.meta.url)));
const labels = Array.from({length: 129}, (_, index) => `label ${String(index).padStart(3, '0')} Å`);
function counterpart(id) {
 const source = fs.readFileSync(new URL(`./workloads/${id}.ts`, import.meta.url), 'utf8');
 const context = vm.createContext({TextEncoder, TextDecoder, console,require:createRequire(import.meta.url)});
 vm.runInContext(ts.transpileModule(source+(/^(higher_|declared_)/.test(id)?'\n'+fs.readFileSync(new URL('declared-value-observation.ts',import.meta.url),'utf8'):'')+(id.startsWith('array_concat_')?'\n'+fs.readFileSync(new URL('concat-observation.ts',import.meta.url),'utf8'):''), {compilerOptions: {target: ts.ScriptTarget.ES2022}}).outputText, context);
 return {call: (...args) => context.workload(...args), context};
}
function digest(values) {
 let result = 0;
 for (const value of values) {
  for (const scalar of value) result = (result * 131 + scalar.codePointAt(0)) % 1000000007;
  result = (result * 131 + 255) % 1000000007;
 } return result;
}
function scalarWidth(text) { return Array.from(text).length; }
for (const leaf of ['int', 'boolean', 'string']) for (const method of ['includes', 'index_of']) {
 test(`${leaf} ${method} oracle agrees with independent dense-array traversal`, () => {
  for (const seed of [0, 3, 7, 19, 30, 64, 97]) {
   const values = leaf === 'string' ? labels : Array.from({length: 129}, (_, index) => {
    const number = (37 * index + seed) % 101;
    return leaf === 'int' ? number : number < 100;
   });
   for (const n of [0, 1, 2, 17, 137, 1000]) {
    let sum = seed;
    for (let index = 0; index < n; index++) {
     const ordinal = (17 * index + seed) % 137;
     const needle = leaf === 'string' ? ordinal < 129 ? labels[ordinal] : 'label 064 Å' : leaf === 'boolean' ? (index + seed) % 2 === 0 : ordinal;
     const from = (13 * index + seed) % 387 - 193;
     const start = from < 0 ? Math.max(0, values.length + from) : from;
     let found = -1;
     for (let position = start; position < values.length; position++) {
      if (values[position] === needle) { found = position; break; }
     }
     sum += method === 'includes' ? Number(found >= 0) : found;
    }
    assert.equal(expectedChecksum(`${leaf}_array_${method}`, n, seed, leaf === 'string' ? 'label 064 Å' : undefined), sum);
   }
  }
 });
}
test('external string search oracle refuses an unspecified input', () => {
 assert.throws(() => expectedChecksum('string_array_includes', 2, 7, 'missing'), /specification changed/);
});
for (const method of ['includes', 'index_of']) {
 test(`dynamic string ${method} oracle agrees with independent content traversal`, () => {
  for (const seed of [0, 3, 7, 19, 30, 64, 97]) {
   const input = '  Label Å  ';
   const values = Array.from({length: 129}, (_, j) => (j * 37 + seed) % 101 < 100 ? input.trim().toLowerCase() : input.trim());
   for (const n of [0, 1, 2, 17, 137, 1000]) {
    let sum = seed;
    for (let i = 0; i < n; i++) {
     const needle = ['label å', 'Label Å', 'label ö'][(i + seed) % 3];
     const from = (i * 13 + seed) % 387 - 193;
     const start = from < 0 ? Math.max(0, values.length + from) : from;
     let found = -1;
     for (let j = start; j < values.length; j++) if (values[j] === needle) { found = j; break; }
     sum += method === 'includes' ? Number(found >= 0) : found;
    }
    assert.equal(expectedChecksum(`dynamic_string_array_${method}`, n, seed, input), sum);
   }
  }
  assert.throws(() => expectedChecksum(`dynamic_string_array_${method}`, 2, 7, 'wrong'), /specification changed/);
 });
}

test('UTF8 length and scalar coordinates are independently distinguished', () => {
 const text = 'Aé😀Z';
 assert.deepEqual(unicodeScalars(text), [65,233,0x1f600,90]);
 assert.deepEqual(byteBoundaries(unicodeScalars(text)), [0,1,3,7,8]);
 assert.throws(() => unicodeScalars('\uD800'), /valid Unicode/);
 assert.throws(() => unicodeScalars('\uDC00'), /valid Unicode/);
});
for (const id of ['string_length','string_slice_unicode','string_char_at','string_char_at_unicode_tail','string_char_at_unicode_late','string_char_at_unicode_middle']) {
 test(`${id} independent byte/scalar oracle checks actual authored Node counterpart`, () => {
  const {call} = counterpart(id), row = manifest.workloads.find(w => w.id === id);
  for (const input of ['', 'Aé😀Z', '\u0085 A😀e\u0301Z \u0085', ' \uFEFFA😀Z\uFEFF ', 'A\0é😀Z']) {
   for (const seed of [0,7,19]) for (const n of [0,1,2,17]) {
    const observed = call(n, seed, input, ...(row.scalarCoordinates ? [scalarWidth(input)] : []));
    assert.equal(expectedChecksum(id,n,seed,input), id === 'string_length' ? observed : digest(observed));
    if (Array.isArray(observed)) assert.ok(observed.every(value => value.isWellFormed()));
   }
  }
 });
}
for (const id of ['string_slice_ascii','string_char_at_ascii_long']) {
 test(`${id} ASCII operations match standard Node methods`, () => {
  const {call} = counterpart(id), row = manifest.workloads.find(w => w.id === id);
  for (const input of ['', 'abc ABC 0123']) for (const seed of [0,7,19]) for (const n of [0,1,2,17]) assert.equal(expectedChecksum(id,n,seed,input),digest(call(n,seed,input,...(row.scalarCoordinates ? [input.length] : []))));
  assert.throws(() => expectedChecksum(id,1,7,'é'), /ASCII/);
 });
}
test('Unicode byte slicing rejects partial boundaries and accepts empty/clamped ranges', () => {
 const {context} = counterpart('string_slice_unicode');
 const slice = (start,end) => context.utf8Slice('Aé😀Z',start,end);
 assert.equal(slice(1,7),'é😀');assert.equal(slice(-7,-1),'é😀');assert.equal(slice(-100,100),'Aé😀Z');
 assert.equal(slice(2,2),'');assert.equal(slice(6,2),'');assert.equal(slice(100,100),'');
 for (const [start,end] of [[2,3],[3,6],[4,7],[-6,-1]]) assert.throws(() => slice(start,end));
});
test('Rust whitespace semantics preserve FEFF and trim0085', () => {
 const {context} = counterpart('string_length');
 assert.equal(context.rustTrim('\uFEFFA\uFEFF'),'\uFEFFA\uFEFF');
 assert.equal(context.rustTrim('\u0085A\u0085'),'A');
 assert.notEqual(expectedChecksum('string_length',2,0,'\uFEFFA\uFEFF'),expectedChecksum('string_length',2,0,'\u0085A\u0085'));
});
for (const method of ['includes','index_of']) {
 test(`complete-scalar ${method} oracle preserves astral and empty needles`, () => {
  const id=`unit_string_array_${method}`,{call}=counterpart(id);
  for (const input of ['A😀e\u0301Z','é😀A','A\0😀Z']) for (const seed of [0,7,19]) for (const n of [0,1,2,17,137]) assert.equal(expectedChecksum(id,n,seed,input),call(n,seed,input,scalarWidth(input)));
  assert.throws(() => expectedChecksum(id,1,7,''), /nonempty/);
 });
}
for (const kind of ['ascii','unicode']) {
 const id=`string_construct_slice_${kind}_long`;
 test(`${kind} construction independently validates matching UTF8-byte widths`, () => {
  const row=manifest.workloads.find(w=>w.id===id),{call}=counterpart(id);
  assert.equal(new TextEncoder().encode(row.inputText).length,3088);
  for (const [n,seed] of [[0,0],[1,7],[2,19],[17,3],[128,7]]) {
   const result=call(n,seed,row.inputText);
   assert.ok(result.every(value=>{const bytes=new TextEncoder().encode(value).length;return bytes>=3073&&bytes<=3087&&value.isWellFormed()}));
   assert.equal(expectedChecksum(id,n,seed,row.inputText),digest(result));
   if(n===2){assert.notEqual(digest(result.slice(1)),digest(result));assert.notEqual(digest([...result].reverse()),digest(result));}
  }
  for(const n of[-1,2001,.5,NaN])assert.throws(()=>expectedChecksum(id,n,7,row.inputText),/range/);
  assert.throws(()=>expectedChecksum(id,1,7,row.inputText+'A'),/specification/);
 });
}
test('literal midpoint validates both source payloads and complete scalars', () => {
 const id='string_char_at_unicode_middle_literal',row=manifest.workloads.find(w=>w.id===id),{call}=counterpart(id);
 const texts=['ts','tsr'].map(ext=>JSON.parse(fs.readFileSync(new URL(`./workloads/${id}.${ext}`,import.meta.url),'utf8').match(/const input = ("[^"\n]*");/)[1]));
 assert.equal(texts[0],texts[1]);assert.deepEqual(unicodeScalars(texts[0]),literalMiddleScalars());assert.equal(scalarWidth(texts[0]),row.literalScalarCount);
 for(const [n,seed]of[[0,0],[1,7],[2,19],[17,3],[128,7]]){
  const result=call(n,seed,row.literalScalarCount);assert.equal(expectedChecksum(id,n,seed),digest(result));
  assert.ok(result.every(value=>Array.from(value).length===1&&value.isWellFormed()));
  if(n>=17)for(const value of['A','😀','é','Z'])assert.ok(result.includes(value));
 }
 assert.throws(()=>expectedChecksum(id,2049,7),/count/);assert.throws(()=>expectedChecksum(id,1,7,'input'),/no external/);
});

test('dynamic clone and normalization independently validate general scalar and Rust-whitespace inputs', () => {
 for(const input of ['', 'A😀é\u0000Z', '\uFEFFABC\u0085']) for(const n of [0,1,2,17]) {
  const {call:actual}=counterpart('dynamic_string_clone');
  assert.equal(digest(actual(n,7,input)),expectedChecksum('dynamic_string_clone',n,7,input));
 }
 for(const input of ['', ' ABC ', '\u0085ABC\u0085', '\uFEFFABC\uFEFF']) for(const n of [0,1,2,17]) {
  const {call:actual}=counterpart('dynamic_normalization');
  assert.equal(digest(actual(n,7,input)),expectedChecksum('dynamic_normalization',n,7,input));
 }
});

test('all current authored counterparts agree with independent results under the current contract', () => {
 for(const row of manifest.workloads.filter(row=>!isApproximate(row))) {
  const source=fs.readFileSync(new URL(`workloads/${row.id}.ts`,import.meta.url),'utf8');
  const driver=adaptTSDriver(fs.readFileSync(new URL('driver.ts',import.meta.url),'utf8'),row);
  // Compile Node imports for the isolated VM harness; require resolves the
  // actual runtime module, including assertions that must execute.
  const code=ts.transpileModule(source+'\n'+driver,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText;
  for(const [n,seed] of [[0,0],[1,7],[2,19],[17,3]]) {
   const printed=[];
   vm.runInNewContext(code,{TextEncoder,TextDecoder,exports:{},require:createRequire(import.meta.url),console:{log:value=>printed.push(JSON.parse(value))},process:{argv:['node','test',String(n),String(seed),'0','1',row.inputText??'']},performance:{now:()=>0}});
   assert.equal(printed.length,1,row.id);
   const observed=printed[0];
   if(requiresExactResult(row)) {
    const validate=isCopying(row)?validateCopyingObservation:isRange(row)?validateRangeObservation:isHigher(row)?validateHigherObservation:isDeclared(row)?validateDeclaredObservation:isConcat(row)?validateConcatObservation:validateExactObservation;
    try { validate(observed,expectedExactResult(row.id,n,seed,row.inputText),row); } catch(error) { throw new Error(row.id+': '+error.message,{cause:error}); }
   }
   assert.equal(observed.checksum,expectedChecksum(row.id,n,seed,row.inputText),`${row.id}: n=${n} seed=${seed}`);
  }
 }
});

test('approximate Math counterparts use immutable references rather than exact Number oracles', () => {
 const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
 for(const row of manifest.workloads.filter(isApproximate)) {
  const tsSource=fs.readFileSync(new URL(`workloads/${row.id}.ts`,import.meta.url),'utf8');
  const nativeSource=fs.readFileSync(new URL(`workloads/${row.id}.tsr`,import.meta.url),'utf8');
  const loaded=loadApproximateReference(row,{tsSha256:hash(tsSource),tsrSha256:hash(nativeSource)});
  // Executes TS point probes; native ABI has its own real-emission suite.
  const arity=row.approximateValidation.operationArity;
  const header=`fn function1(${Array.from({length:arity},(_,i)=>`value${i}: f64`).join(', ')}) -> f64 {} fn main() {}`;
  const probe=approximatePointProbeSources(header,tsSource,row,loaded.reference);
  const points=[];
  vm.runInNewContext(ts.transpileModule(probe.ts,{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText,{console:{log:value=>points.push(JSON.parse(value))}});
  validateApproximatePointProbe(points[0],loaded.reference);
  for(const [n,seed] of [[0,0],[1,7],[2,19],[17,3]]) {
   const printed=[];
   const driver=adaptTSDriver('',row);
   vm.runInNewContext(ts.transpileModule(tsSource+'\n'+driver,{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText,{console:{log:value=>printed.push(JSON.parse(value))},process:{argv:['node','test',String(n),String(seed),'0','1']},performance:{now:()=>0}});
   validateApproximateMeasurement(printed[0],loaded.reference,{iterations:n,seed,warmups:0,samples:1,pointProbeAccepted:true});
  }
 }
});

for(const [id,inputs] of [
 ['string_equality',['','alpha','alpha\0','é','e\u0301','😀','missing']],
 ['string_ordering_ascii',['','alpha','alph','alpha\0','a','z','missing']],
]) test(`${id} independent sequence oracle checks all source comparisons`,()=>{
 const {call}=counterpart(id);
 for(const input of inputs) for(const n of [0,1,2,17]) for(const seed of [0,7,11]) {
  assert.equal(call(n,seed,input),expectedChecksum(id,n,seed,input),`${id}:${n}:${seed}:${JSON.stringify(input)}`);
 }
 for(const input of ['\uD800','\uDC00','\uD800x','x\uDC00']) assert.throws(()=>expectedChecksum(id,0,7,input),/valid Unicode/);
 for(const [n,seed] of [[-1,7],[1000001,7],[17,-1],[17,101]]) assert.throws(()=>expectedChecksum(id,n,seed,'alpha'),/bounded/);
 if(id==='string_ordering_ascii') assert.throws(()=>expectedChecksum(id,2,7,'é'),/ASCII/);
});

for(const id of ['literal_ascii_uppercase','dynamic_unicode_uppercase']) test(`${id} explicit scalar oracle checks uppercase expansion and reusable inputs`,()=>{
 const {call}=counterpart(id);
 const inputs=id==='literal_ascii_uppercase'?[undefined]:['','abcXYZ\0','ß','ﬀﬁﬂ','e\u0301 😀\0','Straße ﬀi e\u0301 😀\0Z','éÉ'];
 for(const input of inputs) for(const n of [0,1,2,17]) for(const seed of [0,7,19]) {
  const before=input, args=input===undefined?[n,seed]:[n,seed,input];
  const result=call(...args);
  assert.equal(digest(result),expectedChecksum(id,n,seed,input));
  assert.equal(input,before); assert.equal(result.length,n);
 }
 for(const [n,seed] of [[-1,7],[1000001,7],[17,-1],[17,101]]) assert.throws(()=>expectedChecksum(id,n,seed,id==='dynamic_unicode_uppercase'?'abc':undefined),/Bounded/);
 if(id==='dynamic_unicode_uppercase') {
  for(const input of ['\uD800','\uDC00','\uD800x','x\uDC00']) assert.throws(()=>expectedChecksum(id,0,7,input),/Valid Unicode/);
  assert.throws(()=>expectedChecksum(id,2,7,'α'),/domain/);
 } else assert.throws(()=>expectedChecksum(id,2,7,'input'),/no external/);
});

for(const id of ['fixed_int_array_reverse','growable_int_array_reverse']) test(`${id} independent sequence observes every reversed position`,()=>{
 const {call}=counterpart(id);
 for(const n of [0,1,2,17,137]) for(const seed of [0,7,19]) assert.equal(call(n,seed),expectedChecksum(id,n,seed));
 const forward=Array.from({length:9},(_,index)=>7+index).reduce((sum,value)=>(sum*131+value)%1000000007,7);
 assert.notEqual(call(1,7),forward);
 assert.throws(()=>expectedChecksum(id,2,7,'input'),/no external/);
 assert.throws(()=>expectedChecksum(id,-1,7),/Bounded/);
});

test('owned string reversal validates each intermediate order by actual scalar content',()=>{
 const {call}=counterpart('owned_string_array_reverse');
 for(const input of ['', ' AbCé😀 ', 'É', 'omega', '😀', 'ALPHA', 'A\0😀éZ']) for(const n of [0,1,2,3,17]) for(const seed of [0,7,19]) {
  const before=input,result=call(n,seed,input);
  assert.equal(result,expectedChecksum('owned_string_array_reverse',n,seed,input));
  assert.equal(input,before);
 }
 // Two reversals restore the final table, but every intermediate table is observed.
 assert.notEqual(call(2,7,'different'),7);
 for(const input of ['\uD800','\uDC00']) assert.throws(()=>expectedChecksum('owned_string_array_reverse',0,7,input),/valid Unicode/);
});

for(const kind of ['ascii','unicode']) for(const method of ['starts_with','ends_with','includes','index_of']) {
 const id=`string_${method}_${kind}`;
 test(`${id} independently observes byte results and omitted/clamped boundary positions`,()=>{
  const {call}=counterpart(id),row=manifest.workloads.find(w=>w.id===id);
  const inputs=kind==='ascii'?['',row.inputText,'A\0alpha\0Z']:['',row.inputText,'Bö🚀Be\u0301🚀Bö🚀Y','Aé😀Ae\u0301😀Aé😀\0'];
  for(const input of inputs) for(const n of [0,1,2,17,137]) for(const seed of [0,7,19]) assert.equal(call(n,seed,input),expectedChecksum(id,n,seed,input));
  for(const input of ['\uD800','x\uDC00'])assert.throws(()=>expectedChecksum(id,0,7,input),/valid Unicode/);
  for(const [n,seed]of[[-1,7],[1000001,7],[2,-1],[2,101]])assert.throws(()=>expectedChecksum(id,n,seed,row.inputText),/Bounded/);
  if(kind==='ascii')assert.throws(()=>expectedChecksum(id,0,7,'é'),/ASCII/);
  else assert.throws(()=>expectedChecksum(id,0,7,'😀'),/queried.*boundary/);
 });
}

test('Unicode counterpart built-ins agree with byte sequence oracle including boundary failures',()=>{
 for(const method of ['starts_with','ends_with','includes','index_of']) {
  const {context}=counterpart(`string_${method}_unicode`);
  for(const input of ['', 'Aé😀Ae\u0301😀Aé😀Z','x\0😀x','ééé','e\u0301é']) {
   const [byteToUnit,unitToByte]=context.coordinates(input),boundaries=byteBoundaries(unicodeScalars(input));
   for(const needle of ['', 'é','e\u0301','😀','x\0','missing']) for(const position of [undefined,-99,999,...boundaries]) {
    assert.equal(context.searchResult(input,needle,position,byteToUnit,unitToByte),expectedStringSearchResult(method,input,needle,position));
   }
   for(let position=0;position<boundaries.at(-1);position++)if(!boundaries.includes(position)) {
    for(const needle of ['', '😀']) {
     assert.throws(()=>context.searchResult(input,needle,position,byteToUnit,unitToByte),/boundary/);
     assert.throws(()=>expectedStringSearchResult(method,input,needle,position),/boundary/);
    }
   }
  }
 }
 assert.equal(expectedStringSearchResult('index_of','Aé😀A','😀'),3);
 assert.equal(expectedStringSearchResult('index_of','Aé😀A','',999),8);
 assert.equal(expectedStringSearchResult('starts_with','e\u0301','é'),false);
 assert.throws(()=>expectedStringSearchResult('includes','A','\uD800'),/valid Unicode/);
});

for(const id of ['classic_for_control','do_while_control','else_if_control']) test(`${id} independently validates observed control tails and first-match branches`,()=>{
 const {call}=counterpart(id);
 for(const n of [0,1,2,17,137,1024])for(const seed of [0,1,3,7,19,23,97])assert.equal(call(n,seed),expectedChecksum(id,n,seed));
 for(const [n,seed]of[[-1,7],[1000001,7],[2,-1],[2,101]])assert.throws(()=>expectedChecksum(id,n,seed),/Bounded/);
});
test('classic zero-trip and post-tested mandatory first iteration remain observable',()=>{
 assert.equal(counterpart('classic_for_control').call(1,0),0);
 assert.equal(counterpart('do_while_control').call(1,0),1);
});
for(const id of ['int_divide','int_negate','int_less_equal','int_greater_equal','int_not_equal','boolean_equal','boolean_not_equal','boolean_or','boolean_short_circuit'])test(`${id} agrees with exact-integer and truth-table reference`,()=>{
 const {call}=counterpart(id);
 for(const n of [0,1,2,3,4,17,33,64,257])for(const seed of [0,1,2,7,19,1000,1000000])assert.equal(call(n,seed),expectedChecksum(id,n,seed));
 for(const [n,seed]of[[-1,0],[0,-1],[1000001,0],[0,1000001],[.5,0]])assert.throws(()=>expectedChecksum(id,n,seed),/domain/);
});
test('compound short circuit observes exactly one correctly tagged RHS call each iteration',()=>{
 const {context,call}=counterpart('boolean_short_circuit'),original=context.mark,observed=[];
 context.mark=(probe,value,tag)=>{observed.push({value,tag});return original(probe,value,tag);};
 assert.equal(call(5,1),expectedChecksum('boolean_short_circuit',5,1));
 assert.deepEqual(observed,[{value:true,tag:1},{value:false,tag:2},{value:true,tag:2},{value:false,tag:1},{value:true,tag:1}]);
});
