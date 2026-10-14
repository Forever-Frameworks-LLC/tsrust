import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {containsCodeSnippet, replaceCodeSnippet} from '../runtime/source-contract-tokens.mjs';
const manifest=JSON.parse(fs.readFileSync(new URL('./pool-manifest.json',import.meta.url),'utf8'));
const hash=source=>createHash('sha256').update(source).digest('hex');
function check(source,expected,kind){
  assert.equal(hash(source),expected);
  for(const snippet of ['let checksum=0;', 'for(let index=0;index<8;index+=2)', 'checksum+=left+right;', 'return checksum;']) {
    const nativeSnippet=kind==='native' && snippet==='checksum+=left+right;'?'checksum+=a+b;':snippet;
    assert.ok(containsCodeSnippet(source,nativeSnippet),nativeSnippet);
  }
  assert.ok(containsCodeSnippet(source,kind==='native'?'pool.configure({workers:2,capacity:16})':'[0,1].map('));
}
test('pooled workload preserves fingerprints, two workers and eight square results',()=>{
  assert.equal(manifest.expectedChecksum,140);
  for(const kind of ['native','typescript']){
    const source=fs.readFileSync(new URL(kind==='native'?manifest.source:manifest.counterpart,import.meta.url),'utf8');
    check(source,kind==='native'?manifest.sourceSha256:manifest.counterpartSha256,kind);
    assert.throws(()=>check(source+' ',hash(source),kind));
    const modified=replaceCodeSnippet(source,'index<8','index<4');
    assert.throws(()=>check(modified,hash(modified),kind));
  }
});
