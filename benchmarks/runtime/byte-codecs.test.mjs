import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {byteCodecRegistrations} from './byte-codecs-workloads.mjs';
import {expectedByteCodecValues,expectedByteCodecObservation} from './byte-codecs-oracle.mjs';
import {assertByteCodecSource} from './byte-codecs-source-contract.mjs';
const rows=byteCodecRegistrations();
test('codec registrations preserve every operation, size, ABI and independent Node oracle',async()=>{
  assert.equal(rows.length,36);assert.equal(new Set(rows.map(row=>row.id)).size,36);
  for(const row of rows){
    for(const [kind,file] of [['native',row.source],['typescript',row.counterpart]]){
      const source=fs.readFileSync(new URL(file,import.meta.url),'utf8');assertByteCodecSource(source,row,kind);
      assert.throws(()=>assertByteCodecSource(source+' ',row,kind));
    }
    const {workload}=await import(new URL(row.counterpart,import.meta.url));
    for(const n of [0,1,3])for(const seed of [0,7]){
      assert.deepEqual(workload(n,seed,row.inputText),expectedByteCodecValues(row.id,n,seed,row.inputText),row.id);
      const observed=expectedByteCodecObservation(row.id,n,seed,row.inputText);assert.ok(Number.isFinite(observed.checksum));
    }
  }
});
test('Node adapters reject noncanonical base64 and preserve decoder policy',async()=>{
  for(const operation of ['base64_decode','base64url_decode']){
    const source=fs.readFileSync(new URL(`workloads/bytes_${operation}.ts`,import.meta.url),'utf8');
    assert.ok(source.includes('base64 trailing bits'));
    const {base64Decode}=await import(new URL(`workloads/bytes_${operation}.ts`,import.meta.url));
    for(const [text,url] of [['Zh==',false],['Zg',false],['Zg==',true],['Z g==',false],['-_8=',false],['+/8',true]])assert.throws(()=>base64Decode(text,url));
    assert.deepEqual([...base64Decode('Zg==',false)],[102]);assert.deepEqual([...base64Decode('-_8',true)],[251,255]);
  }
  const source=fs.readFileSync(new URL('workloads/bytes_utf16le_decode.ts',import.meta.url),'utf8');
  assert.ok(source.includes('fatal: true, ignoreBOM: true'));
});
