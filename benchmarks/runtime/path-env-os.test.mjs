import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import {assertPathEnvOsSource,pathEnvOsChecksum,pathEnvOsEnvironmentFor,pathEnvOsSpecs} from './path-env-os-contract.mjs';
const rows=JSON.parse(fs.readFileSync(new URL('./manifest.json',import.meta.url))).workloads.filter(row=>row.family==='path-env-os');
test('all seven path/environment/OS workloads retain observed calls and denominators',()=>{
 assert.equal(rows.length,7);
 for(const row of rows)for(const [kind,key,hash] of [['native','source','sourceSha256'],['typescript','counterpart','counterpartSha256']]){
  const source=fs.readFileSync(new URL(row[key],import.meta.url),'utf8');assertPathEnvOsSource(source,row,kind);
  const damaged=replaceCodeSnippet(source,pathEnvOsSpecs[row.id][kind],'const value=0;');assert.notEqual(damaged,source);
  assert.throws(()=>assertPathEnvOsSource(damaged,{...row,[hash]:createHash('sha256').update(damaged).digest('hex')},kind));
 }
});
test('fixed lexical and sanitized environment oracles are independent of implementations',()=>{
 for(const [id,weight] of [['path_parse_parts',29],['path_format_options',19],['env_vars_snapshot',42],['env_get_snapshot',12],['os_parallelism_query',1]])for(const n of [0,1,3])assert.equal(pathEnvOsChecksum(id,n,7,pathEnvOsSpecs[id].input),7+n*weight);
 const base={KEPT:'safe',TSRUST_B03_MISSING:'must disappear'};const env=pathEnvOsEnvironmentFor({family:'path-env-os'},base);
 assert.deepEqual(base,{KEPT:'safe',TSRUST_B03_MISSING:'must disappear'});assert.equal(env.TSRUST_B03_MISSING,undefined);assert.equal(env.TSRUST_B03_TEXT,'héllo 🦀');assert.equal(env.TSRUST_B03_EMPTY,'');
});
