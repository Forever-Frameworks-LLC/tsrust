import {createHash} from 'node:crypto';
export function assertStateSource(source,manifest,engine){
 const expected=engine==='native'?manifest.sourceSha256:manifest.counterpartSha256;
 if(createHash('sha256').update(source).digest('hex')!==expected)throw new Error('Retained-state source fingerprint changed.');
 for(const pattern of [/index\s*<\s*128/,/warmup\s*<\s*8/,/sample\s*<\s*31/,/config\.body/])if(!pattern.test(source))throw new Error('Retained-state workload contract changed.');
}
