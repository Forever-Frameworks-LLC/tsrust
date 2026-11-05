import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {codecOperations, codecSizes} from './byte-codecs-workloads.mjs';
export function assertByteCodecSource(source,row,kind) {
  if(!['native','typescript'].includes(kind)||row.family!=='byte-codecs'||!codecOperations.some(operation=>row.id===`bytes_${operation}_${row.inputText}`)||!codecSizes[row.inputText])throw Error('Byte codec identity');
  if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('Byte codec fingerprint');
  if(row.operationsPerIteration!==1||row.resultEncoding!=='array-shape-scalar-v1'||row.result!=='int[]')throw Error('Byte codec denominator');
  for(const code of ['const text = "é😀Ab".repeat(size / 8);','return output;'])if(!containsCodeSnippet(source,code))throw Error('Byte codec corpus/observation');
}
