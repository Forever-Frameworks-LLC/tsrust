import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertStateSource} from './state-contract.mjs';
test('retained handler state has matched registered sources and fixed work',()=>{
 const manifest=JSON.parse(fs.readFileSync(new URL('./state-manifest.json',import.meta.url),'utf8'));
 for(const [file,engine,field] of [['retained-state.tsr','native','sourceSha256'],['retained-state.ts','typescript','counterpartSha256']]){
  const source=fs.readFileSync(new URL(file,import.meta.url),'utf8');assertStateSource(source,manifest,engine);
  const changed=source.replace(/index\s*<\s*128/,'index < 127');assert.throws(()=>assertStateSource(changed,manifest,engine),/fingerprint/);
  assert.throws(()=>assertStateSource(changed,{...manifest,[field]:createHash('sha256').update(changed).digest('hex')},engine),/contract/);
 }
});
