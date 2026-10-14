import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {containsCodeSnippet, replaceCodeSnippet} from '../runtime/source-contract-tokens.mjs';
const hash=source=>createHash('sha256').update(source).digest('hex');
const manifest=JSON.parse(fs.readFileSync(new URL('./channel-manifest.json',import.meta.url),'utf8'));
function validate(source, expectedHash) {
  assert.equal(hash(source),expectedHash);
  for(const snippet of ['step<250000','value=(value*48271)%2147483647','warmup<8','sample<31']) assert.ok(containsCodeSnippet(source,snippet),snippet);
}
test('channel throughput preserves matched computation and measurement protocol',()=>{
  for(const [sourceKey,hashKey] of [['source','sourceSha256'],['counterpart','counterpartSha256']]){
    const source=fs.readFileSync(new URL(manifest[sourceKey],import.meta.url),'utf8');
    validate(source,manifest[hashKey]);
    assert.throws(()=>validate(source+' ',manifest[hashKey]));
    const mutation=replaceCodeSnippet(source,'step<250000','step<1000');
    assert.throws(()=>validate(mutation,hash(mutation)));
  }
  assert.deepEqual(manifest.workerCounts,[0,2,4]);
});
