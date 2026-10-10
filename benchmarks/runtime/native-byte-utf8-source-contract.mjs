import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertNativeByteUtf8Source(source,row,kind){
    if(!['native','typescript'].includes(kind)||row.id!=='native_byte_utf8'||row.family!=='native-byte-utf8')throw Error('Native byte UTF-8 family');
    if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Native byte UTF-8 fingerprint');
    if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One UTF-8 round trip and strict validation scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Native byte UTF-8 denominator');
    const shared=['return "ASCII";', 'return "é😀";', 'return "A\\u0000B";', 'return "\\uFEFFz";', 'const text=textFor((i+seed)%4);','const bad=invalid([65,195,40]);','const truncated=invalid([65,226,130]);','return output;'];
    const native=['match(fromUtf8(text))','match(toUtf8(bytes))','if(decoded===text) { equal=1; }','output.push([before,equal,bad[0],bad[1],truncated[0],truncated[1],length(bytes)]);'];
    const ts=['const constructed=fromArray(values);', '!Number.isInteger(values[i])||values[i]<0||values[i]>255', 'new TextDecoder("utf-8",{fatal:true,ignoreBOM:true})','encoder.encode(text)','decoder.decode(bytes)','offset:firstInvalidOffset(bytes)','output.push([before,equal,bad[0],bad[1],truncated[0],truncated[1],bytes.length]);'];
    for(const text of [...shared,...(kind==='native'?native:ts)])if(!containsCodeSnippet(source,text))throw Error('Native byte UTF-8 scenario: '+text);
}
