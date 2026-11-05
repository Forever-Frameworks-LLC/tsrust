// Matched direct Node built-ins; integer expected bytes are computed independently.
import assert from 'node:assert/strict';
import {performance} from 'node:perf_hooks';
import {packetVectors,floatVectors,integerBytes,integerValue,byteSequenceIndex,copiedWithin} from '../byte-operations-oracle.mjs';
for(const vector of packetVectors){
 const {value,width,order,signed,bytes}=vector; assert.deepEqual(integerBytes(value,width,order,signed),bytes); assert.equal(integerValue(bytes,order,signed),BigInt(value));
 const output=Buffer.alloc(width+1),suffix=order==='little'?'LE':'BE';
 if(width===8){output[`writeBigInt64${suffix}`](BigInt(value),1);assert.equal(output[`readBigInt64${suffix}`](1),BigInt(value));}
 else {output[`write${signed?'Int':'UInt'}${suffix}`](Number(value),1,width);assert.equal(output[`read${signed?'Int':'UInt'}${suffix}`](1,width),Number(value));}
 assert.deepEqual([...output.subarray(1)],bytes);
}
for(const {width,order,value,bytes} of floatVectors){const buffer=new ArrayBuffer(width+1),view=new DataView(buffer),little=order==='little';view[width===4?'setFloat32':'setFloat64'](1,value,little);assert.deepEqual([...new Uint8Array(buffer).subarray(1)],bytes);assert.ok(Object.is(view[width===4?'getFloat32':'getFloat64'](1,little),width===4?Math.fround(value):value));}
for(const [target,start,end] of [[1,0,4],[0,1,5],[2,2,2],[0,0,5]]){const original=Buffer.from([0,1,2,3,4]);assert.deepEqual([...original.copyWithin(target,start,end)],copiedWithin([0,1,2,3,4],target,start,end));}
for(const [haystack,needle,start] of [[[0,1,2,1,2],[1,2],2],[[1,2],[],2],[[1,2],[3],0]])assert.equal(Buffer.from(haystack).indexOf(Buffer.from(needle),start),byteSequenceIndex(haystack,needle,start));
function profile(operation,size,rounds,run){const durations=[];let sink=0;for(let sample=0;sample<39;sample++){const started=performance.now();for(let round=0;round<rounds;round++)sink+=run();if(sample>=8)durations.push((performance.now()-started)*1e6);}assert.ok(Number.isFinite(sink));console.log(JSON.stringify({operation,size,rounds,durations_ns:durations}));}
const facts=bytes=>bytes.length+(bytes[0]??0)+(bytes.at(-1)??0);
if(!process.argv.includes('--check-only')){
 for(const size of [32,4096,1048576]){const input=Buffer.alloc(size,97),different=Buffer.from(input),target=Buffer.alloc(size+1),overlapping=Buffer.from(input);different[size-1]=98;const shortNeedle=Buffer.from('aaab','ascii');const rounds=Math.max(4,Math.min(2048,4*1048576/size));
 profile('concat',size,rounds,()=>facts(Buffer.concat([input,different])));
 profile('copy_into',size,rounds,()=>{target.set(input,1);return facts(target);});
 profile('copy_within_right',size,rounds,()=>facts(overlapping.copyWithin(1,0,size-1)));
 profile('equals_equal',size,rounds,()=>Number(input.equals(input)));
 profile('equals_last_difference',size,rounds,()=>Number(input.equals(different)));
 profile('compare_last_difference',size,rounds,()=>Math.sign(Buffer.compare(input,different)));
 profile('index_of_tail',size,rounds,()=>different.indexOf(shortNeedle));
 }
 for(const size of [256,4096,65536]){const input=Buffer.alloc(size,97),needle=Buffer.alloc(Math.min(size,512)/2,97);needle[needle.length-1]=98;profile('index_of_repetitive_miss',size,4,()=>input.indexOf(needle));}
 const packet=Buffer.alloc(33);const write=()=>{packet.writeInt16LE(12345,1);packet.writeUInt32BE(12345,3);packet.writeBigInt64LE(12345n,7);packet.writeFloatBE(12345,15);packet.writeDoubleLE(12345,19);return facts(packet);};write();
 const read=()=>packet.readInt16LE(1)+packet.readUInt32BE(3)+Number(packet.readBigInt64LE(7))+packet.readFloatBE(15)+packet.readDoubleLE(19);assert.equal(read(),61725);
 profile('packet_read_unaligned',33,65536,read);profile('packet_write_reuse',33,65536,write);
}
