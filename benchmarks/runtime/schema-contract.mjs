import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
export function schemaChecksum(n,seed,input){
 const weight=input==='"hello"'?6:input==='true'?2:undefined;
 if(weight===undefined||!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||!Number.isSafeInteger(seed+n*weight))throw Error('Schema oracle inputs');
 // Five ASCII output characters plus one fully observed root issue.
 return seed+n*weight;
}
export function assertSchemaSource(source,row,kind){
 if(!['native_schema_string','native_schema_boolean'].includes(row.id)||row.family!=='native-schema'||!['native','typescript'].includes(kind))throw Error('Schema identity');
 if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('Schema fingerprint');
 if(row.iterations!==2000||row.operationsPerIteration!==2||row.inputText!==(row.id==='native_schema_boolean'?'true':'"hello"')||row.result!=='int')throw Error('Schema denominator');
 for(const snippet of ['while(i<n){','schema.safeParse(JSON.parse(input))',"schema.safeParse(JSON.parse('42'))",'error.truncated','error.issues.length!==1',"issue.code!=='type-mismatch'",`issue.expected!=='${row.id==='native_schema_boolean'?'boolean':'string'}'`,"issue.observed!=='number'",'issue.path.length!==0','issue.pathTruncated','checksum+=1;','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Schema operation placement: '+snippet);
 if(row.id==='native_schema_boolean'){if(!containsCodeSnippet(source,kind==='native'?"if(!text){throw 'Expected true';}checksum+=1;":"if(!success.value)throw 'Expected true';checksum+=1;"))throw Error('Schema boolean observation');return;}
 if(!containsCodeSnippet(source,kind==='native'?'checksum+=text.length;':'checksum+=success.value.length;'))throw Error('Schema success observation');
}
