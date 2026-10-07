import test from 'node:test';
import assert from 'node:assert/strict';
import {manifestProtocolFingerprint} from './manifest-contract.mjs';

test('manifest history permits new source fingerprints but freezes measurement and semantics',()=>{
    const rows=[{id:'first',sourceSha256:'old',counterpartSha256:'old-ts',iterations:7,operationsPerIteration:1,samples:11,specification:'observe all values'}, {id:'second',iterations:3}];
    const baseline=manifestProtocolFingerprint(rows);
    assert.equal(manifestProtocolFingerprint(rows.map(row=>({...row,sourceSha256:'formatted',counterpartSha256:'formatted-ts'}))),baseline);
    for(const change of [{iterations:8},{operationsPerIteration:2},{samples:12},{specification:'observe fewer values'}]) {
        assert.notEqual(manifestProtocolFingerprint([{...rows[0],...change},rows[1]]),baseline);
    }
    assert.notEqual(manifestProtocolFingerprint([...rows].reverse()),baseline);
});
