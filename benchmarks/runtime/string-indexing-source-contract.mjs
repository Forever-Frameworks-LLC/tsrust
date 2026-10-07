import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertStringIndexingSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='string_indexing'||row.family!=='string-indexing')throw Error('String indexing family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('String indexing fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete string indexing scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('String indexing denominator');
 const shared=['const letters="abcdefghij";','const k=i+seed;const position=k%10;','}catch(error){later="out";}','output.push([first,pair,later,String(position),String(input)]);','return output;'];
 // The platform returns undefined for a missing position; its counterpart
 // states the range check that the native index performs.
 const own=kind==='native'
  ?['const first=letters[position];','const pair=letters[position]+letters[9-position];','try{later=letters[position+5];}']
  :['const found=text[position];if(found===undefined)throw new RangeError("string index out of range");return found;','const first=at(letters,position);','const pair=at(letters,position)+at(letters,9-position);','try{later=at(letters,position+5);}'];
 for(const text of [...shared,...own])if(!containsCodeSnippet(source, text))throw Error('String indexing scenario: '+text);
 if(kind==='native'&&(containsCodeSnippet(source, 'charAt')||containsCodeSnippet(source, '.at(')))throw Error('String indexing scenario uses a method');
}
