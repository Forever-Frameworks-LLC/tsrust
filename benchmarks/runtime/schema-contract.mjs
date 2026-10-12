import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
export function schemaChecksum(n,seed,input){
 const weight=input==='"hello"'?6:(input==='true'||input==='1.5'||input==='native-int')?2:input==='7'?8:undefined;
 if(weight===undefined||!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||!Number.isSafeInteger(seed+n*weight))throw Error('Schema oracle inputs');
 // Each successful scalar contributes its scenario value, plus one observed root issue.
 return seed+n*weight;
}
export function assertSchemaSource(source,row,kind){
 if(!['native_schema_string','native_schema_boolean','native_schema_int','native_schema_number','native_schema_number_int'].includes(row.id)||row.family!=='native-schema'||!['native','typescript'].includes(kind))throw Error('Schema identity');
 if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('Schema fingerprint');
 if(row.id==='native_schema_number_int'){
  if(row.iterations!==2000||row.operationsPerIteration!==2||row.inputText!=='native-int'||row.result!=='int')throw Error('Schema denominator');
  for(const snippet of ['while(i<n){',kind==='native'?'schema.safeParse(42)':'schema.safeParse(42n)',kind==='native'?'schema.safeParse(9007199254740993)':'schema.safeParse(9007199254740993n)',kind==='native'?"if(text!==42.0){throw 'Expected exact integer';}":"if(success.value!==42)throw 'Expected exact integer';",'error.truncated','error.issues.length!==1',"issue.code!=='number-precision'","issue.expected!=='number'","issue.observed!=='integer'",'issue.path.length!==0','issue.pathTruncated','checksum+=1;','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Schema direct integer operation: '+snippet);
  return;
 }
 if(row.iterations!==2000||row.operationsPerIteration!==2||row.inputText!==(row.id==='native_schema_number'?'1.5':row.id==='native_schema_int'?'7':row.id==='native_schema_boolean'?'true':'"hello"')||row.result!=='int')throw Error('Schema denominator');
 for(const snippet of ['while(i<n){','schema.safeParse(JSON.parse(input))',`schema.safeParse(JSON.parse('${row.id==='native_schema_number'?'true':row.id==='native_schema_int'?'1.5':'42'}'))`,'error.truncated','error.issues.length!==1',`issue.code!=='${row.id==='native_schema_int'?'integer-fractional':'type-mismatch'}'`,`issue.expected!=='${row.id==='native_schema_number'?'number':row.id==='native_schema_int'?'integer':row.id==='native_schema_boolean'?'boolean':'string'}'`,`issue.observed!=='${row.id==='native_schema_number'?'boolean':'number'}'`,'issue.path.length!==0','issue.pathTruncated','checksum+=1;','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Schema operation placement: '+snippet);
 if(row.id==='native_schema_number'){if(!containsCodeSnippet(source,kind==='native'?"if(text!==1.5){throw 'Expected fraction';}checksum+=1;":"if(success.value!==1.5)throw 'Expected fraction';checksum+=1;"))throw Error('Schema number observation');return;}
 if(row.id==='native_schema_int'){if(!containsCodeSnippet(source,kind==='native'?'checksum+=text;':'checksum+=success.value;'))throw Error('Schema integer observation');return;}
 if(row.id==='native_schema_boolean'){if(!containsCodeSnippet(source,kind==='native'?"if(!text){throw 'Expected true';}checksum+=1;":"if(!success.value)throw 'Expected true';checksum+=1;"))throw Error('Schema boolean observation');return;}
 if(!containsCodeSnippet(source,kind==='native'?'checksum+=text.length;':'checksum+=success.value.length;'))throw Error('Schema success observation');
}
