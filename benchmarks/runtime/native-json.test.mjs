import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertNativeJsonSource,jsonChecksum,jsonInput} from './native-json-contract.mjs';
import {expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=file=>fs.readFileSync(new URL(file,import.meta.url),'utf8');
for(const id of ['native_json_roundtrip','native_json_namespace','native_json_scalars','native_json_typeof','native_json_inspection','native_json_views','native_json_refinement','native_json_string_reads','native_json_string_views','native_json_string_projection','native_json_construction','native_json_containers','native_json_typed'])test(`${id} preserves registration, operation count and independent oracle`,()=>{
 const rows=JSON.parse(read('./manifest.json')).workloads.filter(row=>row.id===id);
 assert.equal(rows.length,1);const row=rows[0];
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'json');
 for(const [kind,file] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+file);assertNativeJsonSource(source,row,kind);
  assert.throws(()=>assertNativeJsonSource(source+' ',row,kind));
  assert.throws(()=>assertNativeJsonSource(source,{...row,operationsPerIteration:2},kind));
  const changed=source.replace(/while\s*\(i\s*<\s*n\)/,'while (i < 1)');
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  assert.throws(()=>assertNativeJsonSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
 }
 for(const n of [0,1,2,17,2000])for(const seed of [0,7,19])assert.equal(expectedChecksum(row.id,n,seed,row.inputText),seed+n*(id==='native_json_typed'?42:id==='native_json_containers'?34:id==='native_json_construction'?21:['native_json_scalars','native_json_views'].includes(id)?42:id==='native_json_typeof'?6:id==='native_json_inspection'?2:id==='native_json_refinement'?1:['native_json_string_reads','native_json_string_views','native_json_string_projection'].includes(id)?15:31));
 assert.throws(()=>jsonChecksum(1,0,'invalid'));
 assert.throws(()=>jsonChecksum(-1,0,jsonInput));
 assert.equal(assertABI('fn tsr_result_function0(value0: i64, value1: i64, value2: &String) -> Result<i64, TsrFunctionError0> { todo!() }\nfn main() {}',row,{source:read('./'+row.source)}).fallible,true);
});
