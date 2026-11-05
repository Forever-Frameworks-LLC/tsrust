import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {byteOperationCases} from './byte-operations-workloads.mjs';
export function assertByteOperationSource(source,row,kind){
 const specification=byteOperationCases.find(candidate=>candidate.id===row.id&&candidate.inputText===row.inputText);
 if(!specification||!['native','typescript'].includes(kind)||row.family!=='byte-operations')throw Error('Byte operation identity');
 if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('Byte operation fingerprint');
 if(row.operationsPerIteration!==1||row.result!=='int[]'||row.resultEncoding!=='array-shape-scalar-v1'||row.iterations!==specification.iterations)throw Error('Byte operation denominator');
 if(!containsCodeSnippet(source,'return output;'))throw Error('Byte operation observation');
 const token=specification.operation.startsWith('packet_')?'records':specification.operation.startsWith('index_')?'needle':'bytes';
 if(!source.includes(token))throw Error('Byte operation corpus');
}
