import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertMatchingSource} from './source-contract.mjs';
const manifest=JSON.parse(fs.readFileSync(new URL('./manifest.json',import.meta.url),'utf8'));
test('bounded work queue sources and operation guards agree',()=>{
 for(const [engine,file,field]of [['native',manifest.source,'sourceSha256'],['typescript',manifest.counterpart,'counterpartSha256']]){
  const source=fs.readFileSync(new URL(file,import.meta.url),'utf8');assertMatchingSource(source,manifest,engine);
  const changed=source.replace(/id\s*<=\s*64/,'id <= 63');assert.notEqual(changed,source);
  assert.throws(()=>assertMatchingSource(changed,manifest,engine),/fingerprint/);
  const renamed={...manifest,[field]:createHash('sha256').update(changed).digest('hex')};
  assert.throws(()=>assertMatchingSource(changed,renamed,engine),/operation counts/);
 }
});
