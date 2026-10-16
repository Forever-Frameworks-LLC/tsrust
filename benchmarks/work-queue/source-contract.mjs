import {createHash} from 'node:crypto';
export function assertMatchingSource(source,manifest,engine){
 const expected=engine==='native'?manifest.sourceSha256:manifest.counterpartSha256;
 if(createHash('sha256').update(source).digest('hex')!==expected)throw new Error('Work queue source fingerprint changed.');
 const required=[/id\s*<=\s*64/,/sample\s*<\s*31/,/warmup\s*<\s*8/,/worker\s*<\s*2/,/new\s+(?:Channel|Queue)<Work>\(\s*2\s*\)/];
 if(required.some(pattern=>!pattern.test(source)))throw new Error('Work queue operation counts or capacity changed.');
 if((source.match(/consume\(queue\)/g)??[]).length!==2)throw new Error('Work queue consumer count changed.');
}
