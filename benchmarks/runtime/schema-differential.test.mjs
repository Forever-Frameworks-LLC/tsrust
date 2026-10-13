import test from 'node:test';
import assert from 'node:assert/strict';
import {schemaCases,scenarioModes,nativeSource,referencePaths,nativePaths,normalizedPaths} from './schema-differential-cases.mjs';
test('differential corpus covers implemented schema families and both validation paths',()=>{
 assert.equal(new Set(schemaCases.map(value=>value.id)).size,schemaCases.length);
 for(const id of ['recursive-tree','multiple-refinement-issues','string-codepoints','integer-bounds','nested-array','strict-object','strip-object','presence','validated-default','invalid-default','literal','enum','ordered-union','discriminated-union','readonly-refinement','owned-transform','owned-coercion','array-union','nested-transform','default-transform','retained-extras','readonly-union-fallback','cross-field-refinement','record-values','nested-record','record-transform'])assert(schemaCases.some(value=>value.id===id),id);
 const source=nativeSource();
 for(const [index,scenario]of schemaCases.entries()){
  assert(scenario.inputs.length>=3);assert(source.includes(`function scenario${index}()`));
  assert(source.includes(scenario.native));assert(scenarioModes(scenario).includes('fused'));
  let passes=0,failures=0;
  for(const text of scenario.inputs){const result=scenario.reference().safeParse(JSON.parse(text));if(result.success)passes++;else failures++;}
  assert(failures>0,scenario.id);assert(passes>0,scenario.id);
 }
});
test('path comparison preserves string versus numeric keys and expands reference union errors',()=>{
 const native=nativePaths([{path:[{field:'items'},{index:1}]},{path:[{field:'1'}]}]);
 const reference=referencePaths([{code:'invalid_union',path:[],errors:[[{code:'invalid_type',path:['items',1]}]]},{code:'unrecognized_keys',path:[],keys:['1']}]);
 assert.deepEqual(normalizedPaths(native),normalizedPaths(reference));
 assert.notDeepEqual(normalizedPaths([['1']]),normalizedPaths([[1]]));
 assert.notDeepEqual(normalizedPaths([['items',1]]),normalizedPaths([['items',2]]));
});

test('reference union errors retain every enclosing path',()=>{
 const errors=[{code:'invalid_union',path:['items',2],errors:[[{code:'invalid_union',path:['nested'],errors:[[{code:'invalid_type',path:[1]}]]}],[{code:'unrecognized_keys',path:['data'],keys:['extra']}]]}];
 assert.deepEqual(referencePaths(errors),[['items',2,'nested',1],['items',2,'data','extra']]);
});
