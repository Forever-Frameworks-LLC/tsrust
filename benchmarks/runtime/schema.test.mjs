import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertSchemaSource,schemaChecksum} from './schema-contract.mjs';
import {expectedChecksum} from './oracle.mjs';
import {reportCategory} from './report.mjs';
import {assertABI} from './adapters.mjs';
const read=file=>fs.readFileSync(new URL(file,import.meta.url),'utf8');
for(const id of ['native_schema_string','native_schema_boolean','native_schema_int'])test(`${id} preserves workload, oracle and ABI`,()=>{
 const rows=JSON.parse(read('./manifest.json')).workloads.filter(row=>row.id===id);
 assert.equal(rows.length,1);const row=rows[0];assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'schema');
 for(const [kind,file] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+file);assertSchemaSource(source,row,kind);
  assert.throws(()=>assertSchemaSource(source+' ',row,kind));
  const changed=source.replace(/while\s*\(i\s*<\s*n\)/,'while(i<1)');
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  assert.throws(()=>assertSchemaSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  assert.throws(()=>assertSchemaSource(source,{...row,operationsPerIteration:1},kind));
 }
 for(const n of [0,1,17,2000])for(const seed of [0,7,1000000])assert.equal(expectedChecksum(row.id,n,seed,row.inputText),seed+n*(id==='native_schema_int'?8:id==='native_schema_boolean'?2:6));
 for(const args of [[-1,0,row.inputText],[1,0,'42'],[Number.MAX_SAFE_INTEGER,0,row.inputText]])assert.throws(()=>schemaChecksum(...args));
 assert.equal(assertABI('fn tsr_result_function0(value0: i64, value1: i64, value2: &String) -> Result<i64, TsrFunctionError0> { todo!() }\nfn main() {}',row,{source:read('./'+row.source)}).fallible,true);
});
