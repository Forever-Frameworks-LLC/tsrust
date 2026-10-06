import {createHash} from 'node:crypto';
export function assertArrayCallbackErrorsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='array_callback_errors'||row.family!=='array-callback-errors')throw Error('Array callback error family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Array callback error fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One checked callback scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Array callback error denominator');
 for(const text of ['const values=[i+seed+2,i+seed,i+seed+1]','try{values.map(','throw String(input)','finally{finalized=1;}','values.toSorted(','throw "negative"','values.join(","),sorted.join(",")','return output;'])if(!source.includes(text))throw Error('Array callback scenario: '+text);
}
