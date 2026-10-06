import {createHash} from 'node:crypto';
export function assertNumericStringConversionSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='numeric_string_conversion'||row.family!=='numeric-string-conversion')throw Error('Numeric text family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Numeric text fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete numeric text conversion scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Numeric text denominator');
 for(const text of ['"0b101","0o17"','"-0","inf","1e999","-1e-999"','"0x20000000000003","12tail"','String(Number(text)),String(+text)','String(Number(value))','String(Object.is(Number(text),-0.0))','return output;'])if(!source.includes(text))throw Error('Numeric text scenario: '+text);
}
