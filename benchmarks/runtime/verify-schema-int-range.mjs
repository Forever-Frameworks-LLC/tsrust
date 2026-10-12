// Exact integer-token oracle. BigInt avoids JSON.parse's floating-point rounding.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
assert(process.env.TSRUST,'Set TSRUST to the current compiler');
const min=-(1n<<63n),max=(1n<<63n)-1n;
const values=[min-1n,min,min+1n,-9007199254740993n,-1n,0n,1n,9007199254740993n,max-1n,max,max+1n,(1n<<64n)-1n,1n<<64n];
const expected=values.map(value=>value<min||value>max?'integer-range':value.toString()).join('\n')+'\n';
const directory=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-schema-range-'));
function run(args){const result=spawnSync(process.env.TSRUST,args,{encoding:'utf8',timeout:60000,maxBuffer:16*1024*1024});assert.equal(result.status,0,result.stderr||String(result.error));return result.stdout;}
try{
 const file=path.join(directory,'range.tsr');
 fs.writeFileSync(file,`import {s} from 'std:schema';function main(){const schema=s.int();for(const text of [${values.map(value=>JSON.stringify(value.toString())).join(',')}]){match(JSON.tryParse(text)){Ok(input)=>{match(schema.safeParse(input)){Ok(value)=>{console.log(value);},Err(error)=>{for(const issue of error.issues){console.log(issue.code);}}}},Err(error)=>{console.log(error.code);}}}}`);
 run(['format',file]);const formatted=fs.readFileSync(file,'utf8');run(['format',file]);assert.equal(fs.readFileSync(file,'utf8'),formatted);
 assert.equal(run(['run',file]),expected);
 console.log(JSON.stringify({id:'schema-int-range',cases:values.length,oracle:'BigInt',timed:false,status:'pass'}));
}finally{fs.rmSync(directory,{recursive:true,force:true});}
