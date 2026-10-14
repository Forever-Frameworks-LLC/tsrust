import test from 'node:test';
import {replaceCodeSnippet} from '../runtime/source-contract-tokens.mjs';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertMatchingSource} from './source-contract.mjs';
const manifest=JSON.parse(fs.readFileSync(new URL('./manifest.json',import.meta.url),'utf8'));
test('matching workloads retain fingerprints and all three pattern scenarios',()=>{
  for(const kind of ['native','typescript']){
    const source=fs.readFileSync(new URL(kind==='native'?manifest.source:manifest.counterpart,import.meta.url),'utf8');
    assertMatchingSource(source,manifest,kind);
    assert.throws(()=>assertMatchingSource(source+' ',manifest,kind));
    const changed=replaceCodeSnippet(source, 'checksum+=value+weight+extra;', 'checksum+=value;');
    const key=kind==='native'?'sourceSha256':'counterpartSha256';
    assert.throws(()=>assertMatchingSource(changed,{...manifest,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
});
