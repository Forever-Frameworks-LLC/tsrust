import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
export function schemaChecksum(n,seed,input){
 const weight=(input==='owned-array'||input==='[2,3]')?4:input==='"hello"'?6:(input==='true'||input==='1.5'||input==='native-int'||input==='native-int-array'||input==='null')?2:input==='7'?8:undefined;
 if(weight===undefined||!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||!Number.isSafeInteger(seed+n*weight))throw Error('Schema oracle inputs');
 // Each successful scalar contributes its scenario value, plus one observed root issue.
 return seed+n*weight;
}
export function assertSchemaSource(source,row,kind){
 if(!['native_schema_string','native_schema_boolean','native_schema_int','native_schema_number','native_schema_number_int','native_schema_null','native_schema_string_bounds','native_schema_int_bounds','native_schema_int_exclusive','native_schema_number_bounds','native_schema_owned_array','native_schema_json_array','native_schema_number_array_int'].includes(row.id)||row.family!=='native-schema'||!['native','typescript'].includes(kind))throw Error('Schema identity');
 if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('Schema fingerprint');
 if(row.id==='native_schema_number_array_int'){
  if(row.iterations!==2000||row.operationsPerIteration!==2||row.inputText!=='native-int-array'||row.result!=='int')throw Error('Array conversion denominator');
  for(const snippet of [kind==='native'?'s.array(s.number())':'numberArraySchema','while(i<n){',kind==='native'?'const exact:int[]=[42];':'const exact:bigint[]=[42n];',kind==='native'?'const lossy:int[]=[9007199254740993];':'const lossy:bigint[]=[9007199254740993n];','schema.safeParse(exact)','schema.safeParse(lossy)','error.truncated','error.issues.length!==1',"issue.code!=='number-precision'","issue.expected!=='number'","issue.observed!=='integer'",'issue.path.length!==1','issue.pathTruncated','segment.index!==0','segment.field!==undefined',kind==='native'?"if(value!==42.0){throw 'Expected exact conversion';}":"if(value!==42)throw 'Expected exact conversion';",'checksum+=1;','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Array conversion operation: '+snippet);
  return;
 }
 if(row.id==='native_schema_owned_array'||row.id==='native_schema_json_array'){
  const json=row.id==='native_schema_json_array';
  if(row.iterations!==2000||row.operationsPerIteration!==2||row.inputText!==(json?'[2,3]':'owned-array')||row.result!=='int')throw Error('Array schema denominator');
  for(const snippet of [kind==='native'?'s.array(s.int().min(2))':'arraySchema(2)','while(i<n){',json?'schema.safeParse(JSON.parse(input))':'schema.safeParse([2,3])',json?"schema.safeParse(JSON.parse('[1,0]'))":'schema.safeParse([1,0])','error.truncated','error.issues.length!==2',"issue.code!=='too-small'","issue.expected!=='integer'",json?"issue.observed!=='number'":"issue.observed!=='integer'",'issue.path.length!==1','issue.pathTruncated','segment.index!==index','segment.field!==undefined',kind==='native'?'checksum+=value.length;':'checksum+=success.value.length;','checksum+=1;','index++;','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Array schema operation: '+snippet);
  return;
 }
 if(row.id==='native_schema_int_exclusive'&&!containsCodeSnippet(source,kind==='native'?'s.int().gt(2).lt(10)':'input>2&&input<10'))throw Error('Schema exclusive integer bounds');
 if(row.id==='native_schema_int_bounds'&&!containsCodeSnippet(source,kind==='native'?'s.int().min(2).max(9)':'input>=2&&input<=9'))throw Error('Schema integer bounds');
 if(row.id==='native_schema_string_bounds'&&!containsCodeSnippet(source,kind==='native'?'s.string().min(2).max(6)':'length>=2&&length<=6'))throw Error('Schema bounds');
 if(row.id==='native_schema_number_bounds'){
  if(!containsCodeSnippet(source,kind==='native'?'s.number().min(0.0).max(3.0).gt(1.0).lt(2.0)':'input>1&&input<2'))throw Error('Floating schema bounds');
  if(row.iterations!==2000||row.operationsPerIteration!==2||row.inputText!=='1.5'||row.result!=='int')throw Error('Floating schema denominator');
  for(const snippet of ['while(i<n){','schema.safeParse(JSON.parse(input))',"schema.safeParse(JSON.parse('2.0'))",'error.truncated','error.issues.length!==1',"issue.code!=='too-large'","issue.expected!=='number'","issue.observed!=='number'",'issue.path.length!==0','issue.pathTruncated',kind==='native'?"if(text!==1.5){throw 'Expected fraction';}checksum+=1;":"if(success.value!==1.5)throw 'Expected fraction';checksum+=1;",'checksum+=1;','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Floating schema operation: '+snippet);
  return;
 }
 if(row.id==='native_schema_number_int'){
  if(row.iterations!==2000||row.operationsPerIteration!==2||row.inputText!=='native-int'||row.result!=='int')throw Error('Schema denominator');
  for(const snippet of ['while(i<n){',kind==='native'?'schema.safeParse(42)':'schema.safeParse(42n)',kind==='native'?'schema.safeParse(9007199254740993)':'schema.safeParse(9007199254740993n)',kind==='native'?"if(text!==42.0){throw 'Expected exact integer';}":"if(success.value!==42)throw 'Expected exact integer';",'error.truncated','error.issues.length!==1',"issue.code!=='number-precision'","issue.expected!=='number'","issue.observed!=='integer'",'issue.path.length!==0','issue.pathTruncated','checksum+=1;','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Schema direct integer operation: '+snippet);
  return;
 }
 if(row.iterations!==2000||row.operationsPerIteration!==2||row.inputText!==(row.id==='native_schema_null'?'null':row.id==='native_schema_number'?'1.5':(row.id==='native_schema_int'||row.id==='native_schema_int_bounds'||row.id==='native_schema_int_exclusive')?'7':row.id==='native_schema_boolean'?'true':'"hello"')||row.result!=='int')throw Error('Schema denominator');
 for(const snippet of ['while(i<n){','schema.safeParse(JSON.parse(input))',`schema.safeParse(JSON.parse('${(row.id==='native_schema_int_bounds'||row.id==='native_schema_int_exclusive')?'10':row.id==='native_schema_string_bounds'?'"x"':row.id==='native_schema_number'?'true':row.id==='native_schema_int'?'1.5':'42'}'))`,'error.truncated','error.issues.length!==1',`issue.code!=='${(row.id==='native_schema_int_bounds'||row.id==='native_schema_int_exclusive')?'too-large':row.id==='native_schema_string_bounds'?'too-small':row.id==='native_schema_int'?'integer-fractional':'type-mismatch'}'`,`issue.expected!=='${row.id==='native_schema_null'?'null':row.id==='native_schema_number'?'number':(row.id==='native_schema_int'||row.id==='native_schema_int_bounds'||row.id==='native_schema_int_exclusive')?'integer':row.id==='native_schema_boolean'?'boolean':'string'}'`,`issue.observed!=='${row.id==='native_schema_string_bounds'?'string':row.id==='native_schema_number'?'boolean':'number'}'`,'issue.path.length!==0','issue.pathTruncated','checksum+=1;','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Schema operation placement: '+snippet);
 if(row.id==='native_schema_number'){if(!containsCodeSnippet(source,kind==='native'?"if(text!==1.5){throw 'Expected fraction';}checksum+=1;":"if(success.value!==1.5)throw 'Expected fraction';checksum+=1;"))throw Error('Schema number observation');return;}
 if(row.id==='native_schema_int'||row.id==='native_schema_int_bounds'||row.id==='native_schema_int_exclusive'){if(!containsCodeSnippet(source,kind==='native'?'checksum+=text;':'checksum+=success.value;'))throw Error('Schema integer observation');return;}
 if(row.id==='native_schema_null'){if(!containsCodeSnippet(source,kind==='native'?"if(text!==null){throw 'Expected null';}checksum+=1;":"if(success.value!==null)throw 'Expected null';checksum+=1;"))throw Error('Schema null observation');return;}
 if(row.id==='native_schema_boolean'){if(!containsCodeSnippet(source,kind==='native'?"if(!text){throw 'Expected true';}checksum+=1;":"if(!success.value)throw 'Expected true';checksum+=1;"))throw Error('Schema boolean observation');return;}
 if(!containsCodeSnippet(source,kind==='native'?'checksum+=text.length;':'checksum+=success.value.length;'))throw Error('Schema success observation');
}
