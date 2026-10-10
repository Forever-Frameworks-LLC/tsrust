import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertNativeByteCopyAppendSource(source,row,kind) {
    if(!['native','typescript'].includes(kind)||row.id!=='native_byte_copy_append'||row.family!=='native-byte-copy-append')throw Error('Native byte copy/append family');
    if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Native byte copy/append fingerprint');
    if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One checked byte append and independent copy scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Native byte copy/append denominator');
    const shared=['const add = append;','const duplicate = copy;','const byte = (i + seed) % 256;','const initial = length(bytes);','const preserved = read(bytes,3);','const ownerWrite = write(bytes,0,(byte + 1) % 256);','const independent = read(part,0);','const partWrite = write(part,1,9);','const ownerRetained = read(bytes,1);','output.push([initial,success,invalid,offset,preserved,length(part),ownerWrite,independent,partWrite,ownerRetained,empty,rangeOffset,length(bytes)]);','return output;'];
    const native=['match (fromArray([byte,65]))','match (add(bytes,[255,7]))','match (append(bytes,[8,256]))','match (duplicate(bytes,0,2))','match (copy(bytes,4,4))','match (copy(bytes,0,5))'];
    const ts=['const result = fromArray([byte,65]);','const added = add(bytes,[255,7]);','const failed = append(bytes,[8,256]);','const copied = duplicate(bytes,0,2);','const endCopy = copy(bytes,4,4);','const range = copy(bytes,0,5);','next.set(values,bytes.data.length);','bytes.data.slice(start,end)'];
    for(const text of [...shared,...(kind==='native'?native:ts)])if(!containsCodeSnippet(source,text))throw Error('Native byte copy/append scenario: '+text);
}
