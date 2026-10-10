import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertNativeByteAccessSource(source,row,kind) {
    if (!['native','typescript'].includes(kind) || row.id !== 'native_byte_access' || row.family !== 'native-byte-access') throw Error('Native byte access family');
    if (createHash('sha256').update(source).digest('hex') !== (kind==='native'?row.sourceSha256:row.counterpartSha256)) throw Error('Native byte access fingerprint');
    if (row.iterations !== 2048 || row.operationsPerIteration !== 1 || row.operationUnit !== 'One checked byte access and mutation scenario' || row.resultEncoding !== 'array-shape-scalar-v1') throw Error('Native byte access denominator');
    const shared=['const write = set;', 'const byte = (i + seed) % 256;', 'const changed = (byte + 1) % 256;', 'const initial = read(bytes);', 'const updated = read(bytes);', 'const afterIndex = read(bytes);', 'const afterByte = read(bytes);', 'output.push([initial, updated, success, afterIndex, afterByte, indexCode, indexOffset + 1, byteCode, byteOffset, readCode, readOffset, length(bytes)]);', 'return output;'];
    const native=['import { Bytes, fromArray, at, set, length } from "std:bytes";', 'match (fromArray([0, byte, 255]))', 'match (write(bytes, 1, changed))', 'match (set(bytes, -1, 256))', 'match (set(bytes, 1, 256))', 'match (at(bytes, 3))', 'indexOffset = error.offset ?? -3;', 'byteOffset = error.offset ?? -3;', 'readOffset = error.offset ?? -3;'];
    const typescript=['Uint8Array.from(values)', 'if (index < 0 || index >= bytes.length)', 'if (value < 0 || value > 255)', 'bytes[index] = value;', 'const written = write(bytes, 1, changed);', 'const invalidIndex = set(bytes, -1, 256);', 'const invalidByte = set(bytes, 1, 256);', 'const invalidRead = at(bytes, 3);', 'indexOffset = invalidIndex.error.offset ?? -3;', 'byteOffset = invalidByte.error.offset ?? -3;', 'readOffset = invalidRead.error.offset ?? -3;'];
    for(const text of [...shared,...(kind==='native'?native:typescript)]) if(!containsCodeSnippet(source,text)) throw Error('Native byte access scenario: '+text);
}
