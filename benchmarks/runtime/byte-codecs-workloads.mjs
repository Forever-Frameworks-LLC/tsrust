// Additive registration input; the integrator owns the canonical runtime manifest.
import fs from 'node:fs';
import {createHash} from 'node:crypto';
export const codecOperations = ['hex_encode','hex_decode','base64_encode','base64_decode','base64url_encode','base64url_decode','utf16le_encode','utf16be_encode','utf16le_decode','utf16be_decode','utf8_lossy_valid','utf8_lossy_invalid'];
export const codecSizes = {token:32,'4k':4096,'1m':1048576};
export function byteCodecRegistrations() {
  const hash = name => createHash('sha256').update(fs.readFileSync(new URL(name, import.meta.url))).digest('hex');
  return codecOperations.flatMap(operation => Object.entries(codecSizes).map(([size, bytes]) => ({
    id:`bytes_${operation}_${size}`, family:'byte-codecs', rustReference:`parity/bytes_${operation}.rs`, source:`workloads/bytes_${operation}.tsr`, counterpart:`workloads/bytes_${operation}.ts`,
    result:'int[]', resultEncoding:'array-shape-scalar-v1', parameters:['int','int','string'], inputText:size,
    iterations:size==='token'?2048:size==='4k'?1024:4, operationsPerIteration:1, operationUnit:`One ${operation} call on ${bytes} UTF-8 source bytes, plus three scalar observations`,
    category:'bytes', sourceAccepted:false, nativeSourceAccepted:false, implementationAccepted:false, measured:false, measurementStatus:'pending-byte-codecs',
    specification:'Repeated é😀Ab corpus. One input construction/pre-encoding per invocation is included and amortized across calls; setup is measured separately by the direct profile. Each call retains exact output length and boundary observations. Full vector/malformed-input correctness belongs to byte_codecs controls. No hidden whole-result checksum pass.',
    sourceSha256:hash(`workloads/bytes_${operation}.tsr`),counterpartSha256:hash(`workloads/bytes_${operation}.ts`),
  })));
}
