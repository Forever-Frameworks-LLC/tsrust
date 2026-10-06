import {createHash} from 'node:crypto';
export function assertNumericLiteralsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='numeric_literals'||row.family!=='numeric-literals')throw Error('Numeric literal family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Numeric literal fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete numeric literal scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Numeric literal denominator');
 for(const text of ['const mask=0b1010_0011;','const permissions=0o7_7;','const code=0xDE_AD;','const unit=1_000;','.1_2e+0_1;','0xffff_ffff_ffff_ffff;','String(k&mask)','String(Number(k)+fraction)','return output;'])if(!source.includes(text))throw Error('Numeric literal scenario: '+text);
}
