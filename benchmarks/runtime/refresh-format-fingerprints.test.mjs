import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {refreshFingerprintText} from './refresh-format-fingerprints.mjs';

const hash = source => createHash('sha256').update(source).digest('hex');
const original = 'function main(){console.log("a b",1);}';
const formatted = 'function main() {\n    console.log("a b", 1);\n}\n';
const text = JSON.stringify({workloads: [{id: 'example', source: 'example.tsr', sourceSha256: hash(original), iterations: 7}], samples: 11}, null, 2) + '\n';

test('format fingerprint refresh preserves protocols and accepts only unchanged code tokens', () => {
    const result = refreshFingerprintText(text, () => formatted, () => original);
    assert.equal(result.count, 1);
    assert.equal(result.text, text.replace(hash(original), hash(formatted)));
    assert.equal(JSON.parse(result.text).samples, 11);
    for (const changed of [formatted.replace('1)', '2)'), formatted.replace('"a b"', '"ab"')]) {
        assert.throws(() => refreshFingerprintText(text, () => changed, () => original), /Code tokens changed/);
    }
    assert.throws(() => refreshFingerprintText(text, () => formatted, () => 'other();'), /baseline/);
    assert.throws(() => refreshFingerprintText(text, () => formatted, () => original, ['missing']), /Unknown workload/);
    const unchanged = refreshFingerprintText(text, () => original, () => {throw Error('unused');});
    assert.equal(unchanged.count, 0);
    assert.equal(unchanged.text, text);
});


test('implicit workload paths and selected rows retain exact source identity', () => {
    const manifest=JSON.stringify({workloads:[{id:'example',sourceSha256:hash(original)}]},null,2)+'\n';
    const result=refreshFingerprintText(manifest, filename=>{assert.equal(filename,'workloads/example.tsr');return formatted;},(filename,expected)=>{assert.equal(expected,hash(original));return original;},['example']);
    assert.equal(result.count,1);
    assert.equal(result.text,manifest.replace(hash(original),hash(formatted)));
});


test('automatic refresh preserves comment contents as well as executable tokens',()=>{
    const before='// keep this note\n'+original;
    const after='// keep this note\n'+formatted;
    const manifest=text.replace(hash(original),hash(before));
    assert.equal(refreshFingerprintText(manifest,()=>after,()=>before).count,1);
    assert.throws(()=>refreshFingerprintText(manifest,()=>after.replace('keep this note','different note'),()=>before),/Code tokens changed/);
});
