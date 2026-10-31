import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {environmentID,environmentFor,expectedEnvironmentObservation,assertEnvironmentSource} from './environment-contract.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
const read=name=>fs.readFileSync(new URL(name,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(w=>w.id===environmentID);
test('environment fixture is isolated from caller values',()=>{
 const base={TSRUST_BENCH_ENV_TEXT:'wrong',TSRUST_BENCH_ENV_EMPTY:'wrong',TSRUST_BENCH_ENV_MISSING:'wrong',KEEP:'yes'};
 const child=environmentFor(row,base);assert.equal(child.TSRUST_BENCH_ENV_TEXT,'héllo 🦀');assert.equal(child.TSRUST_BENCH_ENV_EMPTY,'');assert.equal(child.TSRUST_BENCH_ENV_MISSING,undefined);assert.equal(child.KEEP,'yes');assert.equal(base.TSRUST_BENCH_ENV_MISSING,'wrong');
});
test('environment source and complete observations stay registered',()=>{
 assert.equal(reportCategory(row.id),'environment');
 assert.equal(assertABI('fn tsr_result_function0(value0: i64, value1: i64) -> Result<Array<String>,TsrFunctionError0<Record4,String>> { todo!() }\nfn main() {}',row).fallible,true);
 assert.equal(row.measurementStatus,'measured-synchronous-host-family');
 assert.equal(expectedEnvironmentObservation(3,0).exactBits,'t:[s(104,233,108,108,111,32,129408),s(),s(60,109,105,115,115,105,110,103,62)]');
 for(const n of [0,1,3,137,20000])for(const seed of [0,7,1000000]){const expected=expectedEnvironmentObservation(n,seed);assert.deepEqual(expectedExactResult(row.id,n,seed),expected);assert.equal(expectedChecksum(row.id,n,seed),expected.checksum);}
 for(const [kind,file] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+file);assertEnvironmentSource(source,row,kind);
  const changed=replaceCodeSnippet(source,'i = i + 1;','i = i + 2;');const key=kind==='native'?'sourceSha256':'counterpartSha256';
  assert.throws(()=>assertEnvironmentSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
 }
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert(source.includes('assertEnvironmentSource(source'));assert(source.includes('environmentFor(workload'));assert(source.includes("'environment-contract.mjs'"));}
});
