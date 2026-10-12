import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import assert from 'node:assert/strict';
export function measureProcessRSS(command,args,env){
 if(process.platform!=='darwin')throw Error('RSS byte protocol currently requires macOS');
 const result=JSON.parse(execFileSync('python3',[fileURLToPath(new URL('./process-rss.py',import.meta.url)),command,...args],{env,encoding:'utf8',timeout:65000,maxBuffer:16*1024*1024}));
 assert.ok(Number.isSafeInteger(result.peakRssBytes)&&result.peakRssBytes>0);
 assert.equal(result.stderr,'');
 return result;
}
