import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {createHash} from 'node:crypto';
import {assertUrlSource,urlChecksum,urlSpecs} from './url-contract.mjs';
import {expectedChecksum} from './oracle.mjs';
import {reportCategory} from './report.mjs';
import {assertABI} from './adapters.mjs';
const read=file=>fs.readFileSync(new URL(file,import.meta.url),'utf8');
const sha=source=>createHash('sha256').update(source).digest('hex');

for(const [id,spec] of Object.entries(urlSpecs))test(`${id} preserves full observations and independent arithmetic`,()=>{
 const native=read(`./workloads/${id}.tsr`),counterpart=read(`./workloads/${id}.ts`);
 const rows=JSON.parse(read('./manifest.json')).workloads.filter(row=>row.id===id);assert.equal(rows.length,1);const row=rows[0];
 assert.equal(row.measured,false);assert.equal(reportCategory(id),'url');
 assert.equal(assertABI('fn tsr_result_function0(value0: i64, value1: i64, value2: &String) -> Result<i64, TsrFunctionError0> { todo!() }\nfn main() {}',row,{source:native}).fallible,true);
 for(const [kind,source] of [['native',native],['typescript',counterpart]]) {
  assertUrlSource(source,row,kind);
  const changed=source.replace(/while\s*\(i\s*<\s*n\)/,'while(i<1)');
  assert.notEqual(changed,source);
  assert.throws(()=>assertUrlSource(changed,{...row,[kind==='native'?'sourceSha256':'counterpartSha256']:sha(changed)},kind));
  assert.throws(()=>assertUrlSource(source,{...row,operationsPerIteration:2},kind));
 }
 const script=counterpart.replaceAll(': number', '').replaceAll(': string', '')+';workload';
 const workload=vm.runInNewContext(script,{URL,URLSearchParams});
 for(const n of [0,1,17,2000])for(const seed of [0,7,1000000]){assert.equal(workload(n,seed,spec.input),urlChecksum(id,n,seed,spec.input));assert.equal(expectedChecksum(id,n,seed,spec.input),seed+n*spec.weight);}
 for(const args of [[id,-1,0,spec.input],[id,1,0,'wrong'],[id,Number.MAX_SAFE_INTEGER,0,spec.input]])assert.throws(()=>urlChecksum(...args));
});

test('query snapshot observer cannot be removed after refreshing a source fingerprint',()=>{
 const id='native_url_query_snapshot',spec=urlSpecs[id];
 const source=read(`./workloads/${id}.tsr`).replace("rows[1].value !== 'two'",'false');
 const row={id,family:'native-url',iterations:2000,operationsPerIteration:1,result:'int',inputText:spec.input,sourceSha256:sha(source)};
 assert.throws(()=>assertUrlSource(source,row,'native'));
});

test('reused owner stays outside the loop for both authored engines',()=>{
 const id='native_url_reused_setters',row=JSON.parse(read('./manifest.json')).workloads.find(row=>row.id===id);
 for(const [kind,file,parse] of [['native',row.source,'const value = unwrap(parseUrl(input));'],['typescript',row.counterpart,'const value = new URL(input);']]) {
  const source=read('./'+file);assert(source.includes(parse));
  const moved=source.replace(parse,'').replace('while (i < n) {','while (i < n) {'+parse);
  assert.throws(()=>assertUrlSource(moved,{...row,[kind==='native'?'sourceSha256':'counterpartSha256']:sha(moved)},kind));
 }
});
