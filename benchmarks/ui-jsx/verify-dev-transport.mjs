// Observe actual wire bytes independently of the server's encoder.
import assert from 'node:assert/strict';
import http from 'node:http';
import {gunzipSync} from 'node:zlib';
import {createHash} from 'node:crypto';
const base=process.argv[2]||'http://127.0.0.1:4317/';
function request(path,encoding,method='GET'){return new Promise((resolve,reject)=>{const headers=encoding===undefined?{}:{'Accept-Encoding':encoding};const req=http.request(new URL(path,base),{method,headers},response=>{const parts=[];response.on('data',part=>parts.push(part));response.on('end',()=>resolve({status:response.statusCode,headers:response.headers,body:Buffer.concat(parts)}));response.on('error',reject);});req.on('error',reject);req.setTimeout(15000,()=>req.destroy(new Error('Transport timeout')));req.end();});}
const state=JSON.parse((await request('/__tsrust_dev')).body);assert.equal(state.error,'');assert.equal(state.building,false);
const asset='/app_bg.wasm?v='+state.revision;const plain=await request(asset);assert.equal(plain.status,200);assert.equal(plain.headers['content-type'],'application/wasm');assert.equal(plain.headers['content-encoding'],undefined);
let compressed;
for(const [accept,expected] of [['gzip','gzip'],['br',undefined],['GZip;q=0, *;q=1',undefined],['*;q=0, gzip;q=1','gzip'],['gzip;q=0.3, identity;q=1',undefined],['gzip;q=0.8, identity;q=0.2','gzip'],['gzip;q=NaN',undefined]]){
 const response=await request(asset,accept);assert.equal(response.status,200,accept);assert.equal(response.headers['content-encoding'],expected,accept);assert.equal(response.headers.vary,'Accept-Encoding');assert.equal(Number(response.headers['content-length']),response.body.length);assert.equal(response.headers['content-type'],'application/wasm');assert.deepEqual(expected?gunzipSync(response.body):response.body,plain.body,accept);if(expected)compressed=response;
}
assert(compressed.body.length<plain.body.length*.5,'Wasm compression should materially reduce this application');
const head=await request(asset,'gzip','HEAD');assert.equal(head.status,200);assert.equal(head.headers['content-encoding'],'gzip');assert.equal(head.body.length,0);assert.equal(Number(head.headers['content-length']),compressed.body.length);
assert.equal((await request(asset,'gzip;q=0, identity;q=0')).status,406);assert.equal((await request('/app_bg.wasm?v='+String(state.revision+1),'gzip')).status,409);
const js=await request('/app.js','gzip');assert.equal(js.headers['content-encoding'],'gzip');assert(gunzipSync(js.body).toString().includes('instantiateStreaming'));
console.log(JSON.stringify({case:'web-dev-compressed-transport',status:'pass',rawBytes:plain.body.length,gzipBytes:compressed.body.length,sha256:createHash('sha256').update(plain.body).digest('hex'),checks:['exact independent decompression','gzip/identity quality negotiation','wildcards and q=0','MIME preserved','HEAD lengths','no acceptable encoding','mixed revision rejection','streaming loader'],timed:false}));
