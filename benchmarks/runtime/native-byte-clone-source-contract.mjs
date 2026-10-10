import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertNativeByteCloneSource(source,row,kind){
    if(!['native','typescript'].includes(kind)||row.id!=='native_byte_clone'||row.family!=='native-byte-clone')throw Error('Native byte clone family');
    if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Native byte clone fingerprint');
    if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One direct and aggregate independent byte clone scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Native byte clone denominator');
    const shared=['const byte = (i + seed) % 256;','const rows: Holder[] = [{bytes:bytes}];','write(rows[0].bytes,0,(byte + 1) % 256);','write(duplicate,1,7);','output.push([read(duplicate,0),read(duplicate,1),read(saved[0].bytes,0),read(saved[0].bytes,1),read(rows[0].bytes,0),length(duplicate),length(saved[0].bytes)]);','return output;'];
    const native=['match(alloc(3,byte))','const duplicate = clone(bytes);','const saved = clone(rows);'];
    const ts=['const bytes = new Uint8Array(3).fill(byte);','const duplicate = cloneBytes(bytes);','const saved = cloneRows(rows);','return bytes.slice();','return rows.map(row=>({bytes:cloneBytes(row.bytes)}));'];
    for(const text of [...shared,...(kind==='native'?native:ts)])if(!containsCodeSnippet(source,text))throw Error('Native byte clone scenario: '+text);
}
