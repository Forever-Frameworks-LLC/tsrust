import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';
import {byteOperationRegistrations} from './byte-operations-workloads.mjs';
import {expectedByteOperationValues,expectedByteOperationObservation,integerBytes,integerValue,packetVectors,floatVectors,byteSequenceIndex,copiedWithin} from './byte-operations-oracle.mjs';
import {assertByteOperationSource} from './byte-operations-source-contract.mjs';
test('39 byte operation rows keep exact source identities, ABI and independent Node outputs',async()=>{
 const rows=byteOperationRegistrations();assert.equal(rows.length,39);assert.equal(new Set(rows.map(row=>row.id)).size,39);
 for(const row of rows){for(const [kind,file] of [['native',row.source],['typescript',row.counterpart]]){const source=fs.readFileSync(new URL(file,import.meta.url),'utf8');assertByteOperationSource(source,row,kind);assert.throws(()=>assertByteOperationSource(source+' ',row,kind));}
  const {workload}=await import(new URL(row.counterpart,import.meta.url));for(const n of [0,1,3])for(const seed of [0,7]){assert.deepEqual(workload(n,seed,row.inputText),expectedByteOperationValues(row.id,n,seed,row.inputText),row.id);assert.ok(Number.isFinite(expectedByteOperationObservation(row.id,n,seed,row.inputText).checksum));}
 }
});
test('independent exact integer, IEEE and overlap controls preserve explicit bounds',()=>{
 for(const {value,width,order,signed,bytes} of packetVectors){assert.deepEqual(integerBytes(value,width,order,signed),bytes);assert.equal(integerValue(bytes,order,signed),BigInt(value));}
 for(const width of [1,2,4,8])for(const order of ['little','big']){const bits=BigInt(width*8);for(const value of [-(1n<<(bits-1n)),0n,(1n<<(bits-1n))-1n])assert.equal(integerValue(integerBytes(value,width,order,true),order,true),value);}
 for(const [value,width,signed] of [[128,1,true],[-129,1,true],[-1,1,false],[256,1,false],[4294967296,4,false],[0,8,false],[0,3,true]])assert.throws(()=>integerBytes(value,width,'little',signed));
 for(const {width,order,value,bytes} of floatVectors){const buffer=new ArrayBuffer(width+1),view=new DataView(buffer),little=order==='little';view[width===4?'setFloat32':'setFloat64'](1,value,little);assert.deepEqual([...new Uint8Array(buffer).subarray(1)],bytes);assert.ok(Object.is(view[width===4?'getFloat32':'getFloat64'](1,little),width===4?Math.fround(value):value));}
 assert.deepEqual(copiedWithin([0,1,2,3,4],1,0,4),[0,0,1,2,3]);assert.deepEqual(copiedWithin([0,1,2,3,4],0,1,5),[1,2,3,4,4]);assert.throws(()=>copiedWithin([0],1,0,1));assert.equal(byteSequenceIndex([0,255,0,255],[0,255],1),2);assert.equal(byteSequenceIndex([0],[],1),1);assert.throws(()=>byteSequenceIndex([0],[],-1));
});
