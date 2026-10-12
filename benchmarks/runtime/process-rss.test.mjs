import test from 'node:test';
import assert from 'node:assert/strict';
import {measureProcessRSS} from './process-rss.mjs';
test('RSS probe retains stdout and rejects failed children',{skip:process.platform!=='darwin'},()=>{
 const result=measureProcessRSS(process.execPath,['-e','console.log(JSON.stringify({checksum:7}))'],process.env);
 assert.deepEqual(JSON.parse(result.stdout),{checksum:7});
 assert.ok(Number.isSafeInteger(result.peakRssBytes)&&result.peakRssBytes>0);
 assert.throws(()=>measureProcessRSS(process.execPath,['-e','process.exit(3)'],process.env));
});
