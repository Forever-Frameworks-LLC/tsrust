import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertMatchingSource} from './source-contract.mjs';
test('shared cache matches the scalar LRU workload with explicit owned reads',()=>{
 const manifest=JSON.parse(fs.readFileSync(new URL('./shared-manifest.json',import.meta.url),'utf8'));
 for(const [file,engine,key]of [['shared-lru.tsr','native','sourceSha256'],['shared-lru.ts','typescript','counterpartSha256']]){
  const source=fs.readFileSync(new URL(file,import.meta.url),'utf8');assertMatchingSource(source,manifest,engine);
  const changed=source.replace(/i\s*<\s*256/,'i < 255');assert.throws(()=>assertMatchingSource(changed,{...manifest,[key]:createHash('sha256').update(changed).digest('hex')},engine),/scenario/);
 }
});
