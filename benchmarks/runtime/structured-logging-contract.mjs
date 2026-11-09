import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
export const loggingIDs=['logging_disabled','logging_file_output','logging_child_file'];
export function loggingChecksum(n,seed){let sum=0n;for(let i=0n;i<BigInt(n);i++)sum=(sum*131n+i+BigInt(seed))%1000000007n;return Number(sum);}
export function assertLoggingSource(source,row,engine){if(!loggingIDs.includes(row.id)||row.family!=='structured-logging')throw Error('Unknown logging workload');const reference=engine==='rust-hand';const key=reference?'rustReferenceSha256':engine==='native'?'sourceSha256':'counterpartSha256';if(createHash('sha256').update(source).digest('hex')!==row[key])throw Error('Logging source fingerprint changed');for(const snippet of row.sourceRequired[reference?'rust':engine==='native'?'native':'typescript'])if(!containsCodeSnippet(source,snippet))throw Error('Logging operation changed: '+snippet);}
