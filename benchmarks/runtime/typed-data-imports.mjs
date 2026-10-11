import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
const hash=text=>createHash('sha256').update(text).digest('hex');
const input='{"base":5,"values":[3,11,23]}\n';
export function dataImportChecksum(n,seed){
 if(!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||!Number.isSafeInteger(seed+n*42))throw Error('Data import oracle inputs');
 return seed+n*42;
}
export function assertDataImportSource(source,row,kind){
 if(row.id!=='native_json_import'||row.family!=='typed-data-imports'||!['native','typescript'].includes(kind))throw Error('Data import identity');
 if(hash(source)!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('Data import source fingerprint');
 if(row.iterations!==2000||row.operationsPerIteration!==1||row.operationUnit!=='One imported scalar read and three-element array scan'||row.result!=='int'||row.dataSha256!==hash(input))throw Error('Data import operation contract');
 for(const snippet of ["import settings from './typed-data/settings.json';",'let checksum = seed;','let i = 0;','while (i < n) { checksum += settings.base; for (const value of settings.values) { checksum += value; } i++; }','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Data import operation placement');
}
export function prepareDataImport(row,build){
 const data=fs.readFileSync(new URL('./workloads/typed-data/settings.json',import.meta.url));
 if(row.id!=='native_json_import'||hash(data)!==row.dataSha256||data.toString()!==input)throw Error('Data import input fingerprint');
 fs.mkdirSync(path.join(build,'typed-data'),{recursive:true});
 fs.writeFileSync(path.join(build,'typed-data/settings.json'),data);
 return {resolveJsonModule:true,esModuleInterop:true};
}
