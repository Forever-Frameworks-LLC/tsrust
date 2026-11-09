import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import zlib from 'node:zlib';
import {Readable} from 'node:stream';
export function input(size,random,seed=7){let state=seed>>>0;const text=Buffer.from('tsrust-compression-stream\n'),bytes=Buffer.alloc(size);for(let index=0;index<size;index++){if(random){state=(Math.imul(state,1664525)+1013904223)>>>0;bytes[index]=state>>>24;}else bytes[index]=text[index%text.length];}return bytes;}
export function checksum(bytes){let sum=7;for(const byte of bytes)sum=(sum*131+byte)%1000000007;return sum;}
export function decode(format,bytes){return ({gzip:zlib.gunzipSync,zlib:zlib.inflateSync,raw:zlib.inflateRawSync})[format](bytes);}
export function encode(format,bytes,level){return ({gzip:zlib.gzipSync,zlib:zlib.deflateSync,raw:zlib.deflateRawSync})[format](bytes,level===-1?{}:{level});}
export function cases(){return ['gzip','zlib','raw'].flatMap(format=>['writer','reader'].flatMap(strategy=>['compress','decompress'].flatMap(operation=>['compressible','random'].flatMap(pattern=>[0,1,-1,9].flatMap(level=>[64,8192,16384].map(chunk=>({format,strategy,operation,pattern,level,chunk})))))));}
export function corpusName(row){return `${row.format}-${row.pattern}-${row.level}.bin`;}
async function transform(row,bytes){
 const factories=row.operation==='compress'?{gzip:zlib.createGzip,zlib:zlib.createDeflate,raw:zlib.createDeflateRaw}:{gzip:zlib.createGunzip,zlib:zlib.createInflate,raw:zlib.createInflateRaw};
 const codec=factories[row.format](row.level===-1?{}:{level:row.level}),output=[];
 const done=new Promise((resolve,reject)=>{let ended=false;codec.on('data',bytes=>output.push(bytes));codec.once('end',()=>{ended=true;});codec.once('close',()=>{if(ended)resolve(Buffer.concat(output));else reject(Error('Codec closed before complete output'));});codec.once('error',reject);});
 if(row.strategy==='reader')Readable.from((function*(){for(let offset=0;offset<bytes.length;offset+=row.chunk)yield bytes.subarray(offset,offset+row.chunk);})()).pipe(codec);
 else{for(let offset=0;offset<bytes.length;offset+=row.chunk){if(!codec.write(bytes.subarray(offset,offset+row.chunk)))await new Promise((resolve,reject)=>{const failed=error=>{codec.removeListener('drain',drained);reject(error);};const drained=()=>{codec.removeListener('error',failed);resolve();};codec.once('drain',drained);codec.once('error',failed);});}codec.end();}
 return done;
}
const fixtures=process.argv.includes('--fixtures');
if(process.argv.includes('--measure')||fixtures){
 assert.equal(process.version,'v24.11.1');
 const groupIndex=process.argv.indexOf('--group'),group=groupIndex<0?null:process.argv[groupIndex+1];
 if(group!==null)assert(['gzip.compress','gzip.decompress','zlib.compress','zlib.decompress','raw.compress','raw.decompress'].includes(group));
 for(const row of cases()){
  if(fixtures&&row.operation!=='compress')continue;
  if(!fixtures&&group!==null&&`${row.format}.${row.operation}`!==group)continue;
  const original=input(16384,row.pattern==='random');
  const source=row.operation==='compress'?original:fs.readFileSync(path.join(process.env.TSRUST_COMPRESSION_CORPUS??assert.fail('immutable common decode corpus required'),corpusName(row)));
  const expected=await transform(row,source);if(row.operation==='compress')assert.deepEqual(decode(row.format,expected),original);else assert.deepEqual(expected,original);
  const digest=checksum(expected),length=expected.length;
  const label={backend:'node',...row,inputBytes:source.length,inputChecksum:checksum(source),outputBytes:length,checksum:digest};
  if(fixtures){console.log(JSON.stringify({kind:'fixture',...label,hex:expected.toString('hex')}));continue;}
  for(let warmup=0;warmup<8;warmup++)await transform(row,source);
  const durations_ns=[],checksums=[],outputLengths=[];
  for(let sample=0;sample<31;sample++){const start=process.hrtime.bigint(),result=await transform(row,source),observedLength=result.length,observedChecksum=checksum(result);assert.equal(observedLength,length);assert.equal(observedChecksum,digest);durations_ns.push(Number(process.hrtime.bigint()-start));checksums.push(observedChecksum);outputLengths.push(observedLength);}
  console.log(JSON.stringify({kind:'timing',...label,durations_ns,checksums,outputLengths}));
 }
}
