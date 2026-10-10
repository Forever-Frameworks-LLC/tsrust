import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertNativeByteGrowthSource(source,row,kind) {
    if(!['native','typescript'].includes(kind)||row.id!=='native_byte_growth'||row.family!=='native-byte-growth')throw Error('Native byte growth family');
    if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Native byte growth fingerprint');
    if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One checked byte allocation and resize scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Native byte growth denominator');
    const shared=['const change = resize;', 'const byte = (i + seed) % 256;', 'const initial = read(bytes, 0);', 'const zero = read(bytes, 3);', 'const preserved = length(bytes);', 'output.push([initial, growth, zero, invalid, preserved, shrink, regrow, read(bytes, 0), read(bytes, 1), read(bytes, 2), length(bytes)]);', 'return output;'];
    const native=['match (alloc(2, byte))', 'match (change(bytes, 4))', 'match (resize(bytes, 0, 256))', 'match (resize(bytes, 1))', 'match (resize(bytes, 3, 255))'];
    const ts=['const result = alloc(2, byte);','const grown = change(bytes, 4);','const failed = resize(bytes, 0, 256);','const shrunk = resize(bytes, 1);','const regrown = resize(bytes, 3, 255);','bytes.data = next;','next.fill(fill, bytes.data.length);'];
    for(const snippet of [...shared,...(kind==='native'?native:ts)])if(!containsCodeSnippet(source,snippet))throw Error('Native byte growth scenario: '+snippet);
}
