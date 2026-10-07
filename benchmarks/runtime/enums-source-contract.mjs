import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertEnumsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='enum_values'||row.family!=='enums')throw Error('Enum family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Enum fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete enum scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Enum denominator');
 const integer=kind==='native'?'int':'number';
 for(const text of ['enum Phase { Ready, Busy = 4, Done }','enum Label { Short = "s", Long = "long" }',`function workload(n: ${integer}, seed: ${integer}, input: string): string[][]`,'const phase=choose(i+seed);','const copy=phase;','phase===Phase.Ready?Label.Short:Label.Long','String(Number(copy))','input+String(Number(phase))','return output;'])if(!containsCodeSnippet(source, text))throw Error('Enum scenario: '+text);
}
