import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
export function schemaChecksum(n,seed,input){
 const weight=input==='owned-pipeline'?7:input==='\"union-value\"'?13:input==='{\"kind\":\"count\",\"count\":4}'?5:input==='{"label":"ready"}'?9:input==='{"label":null}'?3:input==='{"count":3,"extra":true}'?5:input==='{"count":2,"extra":true}'?3:(input==='owned-array'||input==='[2,3]'||input==='{"count":2}')?4:(input==='"hello"'||input==='"ready"')?6:(input==='2.5'||input==='true'||input==='1.5'||input==='native-int'||input==='native-int-array'||input==='null')?2:input==='7'?8:undefined;
 if(weight===undefined||!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||!Number.isSafeInteger(seed+n*weight))throw Error('Schema oracle inputs');
 // Each successful scalar contributes its scenario value, plus one observed root issue.
 return seed+n*weight;
}
export function assertSchemaSource(source,row,kind){
 if(!['native_schema_string','native_schema_boolean','native_schema_int','native_schema_number','native_schema_number_int','native_schema_null','native_schema_string_bounds','native_schema_int_bounds','native_schema_int_exclusive','native_schema_number_bounds','native_schema_owned_array','native_schema_json_array','native_schema_number_array_int','native_schema_json_fused','native_schema_json_object','native_schema_json_object_strip','native_schema_json_object_retain','native_schema_json_presence','native_schema_json_defaults','native_schema_json_literal','native_schema_json_enum','native_schema_json_declared_enum','native_schema_pipeline','native_schema_json_union','native_schema_json_discriminated_union'].includes(row.id)||row.family!=='native-schema'||!['native','typescript'].includes(kind))throw Error('Schema identity');
 if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('Schema fingerprint');
 if(row.id==='native_schema_json_fused'){
  if(row.operationsPerIteration!==2||!containsCodeSnippet(source,'while(i<n)')||!containsCodeSnippet(source,'i++;'))throw Error('Fused schema loop');
  const required=kind==='native'?['schema.safeParseJson(input)',`schema.safeParseJson('{"extra":true}')`,'checksum+=value.count;','checksum+=1;',`error.kind!=='validation'`,'error.error.issues.length!==2']:['schema.safeParse(JSON.parse(input))','checksum+=success.value.count;','checksum+=1;','error.issues.length!==2'];
  for(const snippet of required)if(!containsCodeSnippet(source,snippet))throw Error('Fused schema observation: '+snippet);
  return;
 }
 if(row.id==='native_schema_pipeline'){
  if(row.iterations!==2000||row.operationsPerIteration!==3||row.result!=='int'||row.inputText!=='owned-pipeline')throw Error('Pipeline denominator');
  for(const snippet of ['while(i<n){','values.push(7);return values.length;','schema.safeParse([2,3])','schema.safeParse([])','value!==4','checksum+=value;','error.truncated','error.issues.length!==1',"issue.code!=='too-small'","issue.expected!=='integer'","issue.observed!=='integer'",'issue.pathTruncated','issue.path.length!==0','checksum+=1;','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Pipeline operation: '+snippet);
  const plan=kind==='native'?['s.int().min(3)', '.coerce((values:int[]):int=>{values.push(7);return values.length;})','.refine(value=>value<10)','.transform(value=>value+1)']:['const value=coerce(input);',"value<3?'too-small':!refine(value)?'refinement-failed':undefined",'value:transform(value)'];
  for(const snippet of plan)if(!containsCodeSnippet(source,snippet))throw Error('Pipeline stage: '+snippet);
  for(const snippet of ["retry.safeParse(JSON.parse('[2,3]'))",...(kind==='native'?['s.array(s.int()).refine(values=>values.length>3).transform(values=>0)','s.array(s.int()).transform(values=>values.length)',"value!==2"]:['const branches=[(values:number[])=>values.length>3,(values:number[])=>true]','for(const [index,accept] of branches.entries())','value:index===0?0:input.length','issues.length=0','restored.value!==2','checksum+=restored.value;'])])if(!containsCodeSnippet(source,snippet))throw Error('Pipeline rollback: '+snippet);
  return;
 }
 if(row.id==='native_schema_json_union'||row.id==='native_schema_json_discriminated_union'){
  const disc=row.id.endsWith('discriminated_union');
  const payload=disc?'{"kind":"count","count":4}':'"union-value"';
  if(row.iterations!==2000||row.operationsPerIteration!==2||row.result!=='int'||row.inputText!==payload)throw Error('Union denominator');
  const invalid=disc?'{"kind":"unknown"}':'true';
  for(const snippet of ['while(i<n){','schema.safeParse(JSON.parse(input))',`schema.safeParse(JSON.parse('${invalid}'))`,'error.truncated',`error.issues.length!==${disc?1:2}`,`issue.code!=='${disc?'discriminator-mismatch':'type-mismatch'}'`,`issue.expected!=='${disc?'union':'string'}'`,`issue.observed!=='${disc?'string':'boolean'}'`,'issue.pathTruncated',`issue.path.length!==${disc?1:0}`,'checksum+=1;','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Union operation: '+snippet);
  for(const snippet of disc?["value.kind==='count'",'value.count!==4','checksum+=value.count;',"segment.field!=='kind'",'segment.index!==undefined']:["value!=='union-value'",'checksum+=value.length;'])if(!containsCodeSnippet(source,snippet))throw Error('Union observation: '+snippet);
  if(kind==='native'&&!containsCodeSnippet(source,disc?"s.discriminatedUnion('kind',[s.object({kind:s.literal('count'),count:s.int()}),s.object({kind:s.literal('text'),text:s.string()})])":"s.union([s.string().min(20),s.string().min(2)])"))throw Error('Union plan');
  if(kind==='typescript'&&!containsCodeSnippet(source,disc?"value.kind==='count'":"for(const min of limits)"))throw Error('Union reference selection');
  return;
 }
 if(row.id==='native_schema_json_declared_enum'){
  if(row.iterations!==2000||row.operationsPerIteration!==2||row.inputText!=='2.5'||row.result!=='int')throw Error('Enum schema denominator');
  for(const snippet of [kind==='native'?"s.enum(Code)":"enumSchema()",'while(i<n){','schema.safeParse(JSON.parse(input))',`schema.safeParse(JSON.parse('3.0'))`,"value!==Code.High",'checksum+=1;','checksum+=1;','error.truncated','error.issues.length!==1',"issue.code!=='literal-mismatch'","issue.expected!=='number'","issue.observed!=='number'",'issue.pathTruncated','issue.path.length!==0','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Enum schema operation: '+snippet);
  if(kind==='typescript'&&!containsCodeSnippet(source,"if(input===Code.Low||input===Code.High)"))throw Error('Enum reference alternatives');
  return;
 }
 if(row.id==='native_schema_json_enum'){
  if(row.iterations!==2000||row.operationsPerIteration!==2||row.inputText!=='"ready"'||row.result!=='int')throw Error('Enum schema denominator');
  for(const snippet of [kind==='native'?"s.enum(['ready','done'])":"enumSchema()",'while(i<n){','schema.safeParse(JSON.parse(input))',`schema.safeParse(JSON.parse('"waiting"'))`,"value!=='ready'",'checksum+=value.length;','checksum+=1;','error.truncated','error.issues.length!==1',"issue.code!=='literal-mismatch'","issue.expected!=='string'","issue.observed!=='string'",'issue.pathTruncated','issue.path.length!==0','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Enum schema operation: '+snippet);
  if(kind==='typescript'&&!containsCodeSnippet(source,"if(input==='ready'||input==='done')"))throw Error('Enum reference alternatives');
  return;
 }
 if(row.id==='native_schema_json_literal'){
  if(row.iterations!==2000||row.operationsPerIteration!==2||row.inputText!=='"ready"'||row.result!=='int')throw Error('Literal schema denominator');
  for(const snippet of [kind==='native'?"s.literal('ready')":"literalSchema()",'while(i<n){','schema.safeParse(JSON.parse(input))',`schema.safeParse(JSON.parse('"waiting"'))`,"value!=='ready'",'checksum+=value.length;','checksum+=1;','error.truncated','error.issues.length!==1',"issue.code!=='literal-mismatch'","issue.expected!=='string'","issue.observed!=='string'",'issue.pathTruncated','issue.path.length!==0','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Literal schema operation: '+snippet);
  if(kind==='typescript'&&!containsCodeSnippet(source,"if(input==='ready')"))throw Error('Literal reference equality');
  return;
 }
 if(row.id==='native_schema_json_defaults'){
  if(kind==='typescript')for(const snippet of ['const factory=()=>7;',"Object.hasOwn(input,'count')?input.count:factory()",'else if(count<5)'])if(!containsCodeSnippet(source,snippet))throw Error('Default reference operation: '+snippet);
  if(row.iterations!==2000||row.operationsPerIteration!==2||row.inputText!=='{"label":"ready"}'||row.result!=='int')throw Error('Presence schema denominator');
  for(const snippet of [kind==='native'?'s.object({count:s.int().default(()=>7).min(5),label:s.string()})':'objectSchema()', 'while(i<n){','schema.safeParse(JSON.parse(input))',`schema.safeParse(JSON.parse('{"count":null}'))`,"value.count!==7||value.label!=='ready'",'error.truncated','error.issues.length!==2','issue.pathTruncated','issue.path.length!==1','segment.index!==undefined',"segment.field==='count'","issue.code!=='type-mismatch'","issue.expected!=='integer'","issue.observed!=='null'","segment.field==='label'","issue.code!=='missing-field'","issue.expected!=='string'","issue.observed!=='missing'",'checksum+=value.count;','checksum+=1;','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Presence schema operation: '+snippet);
  return;
 }
 if(row.id==='native_schema_json_presence'){
  if(row.iterations!==2000||row.operationsPerIteration!==2||row.inputText!=='{"label":null}'||row.result!=='int')throw Error('Presence schema denominator');
  for(const snippet of [kind==='native'?'s.object({count:s.int().optional(),label:s.string().nullable()})':'objectSchema()', 'while(i<n){','schema.safeParse(JSON.parse(input))',`schema.safeParse(JSON.parse('{"count":null}'))`,'value.count!==undefined||value.label!==null','error.truncated','error.issues.length!==2','issue.pathTruncated','issue.path.length!==1','segment.index!==undefined',"segment.field==='count'","issue.code!=='type-mismatch'","issue.expected!=='integer'","issue.observed!=='null'","segment.field==='label'","issue.code!=='missing-field'","issue.expected!=='string'","issue.observed!=='missing'",'checksum+=1;','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Presence schema operation: '+snippet);
  return;
 }
 if(row.id==='native_schema_json_object_retain'){
  if(row.iterations!==2000||row.operationsPerIteration!==2||row.inputText!=='{"count":3,"extra":true}'||row.result!=='int')throw Error('Retained schema denominator');
  for(const snippet of [kind==='native'?'s.object({count:s.int().min(2)}).retain()':'objectSchema()', 'while(i<n){','schema.safeParse(JSON.parse(input))',`schema.safeParse(JSON.parse('{"extra":true}'))`,'error.truncated','error.issues.length!==1','issue.pathTruncated','issue.path.length!==1','segment.index!==undefined',"issue.code==='missing-field'","segment.field!=='count'","issue.expected!=='integer'","issue.observed!=='missing'",kind==='native'?'checksum+=value.value.count+value.extra.size;':'checksum+=success.value.value.count+success.value.extra.size;',kind==='native'?"value.extra.has('extra')":"success.value.extra.has('extra')",kind==='native'?'value.extra.size!==1':'success.value.extra.size!==1','checksum+=1;','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Retained schema operation: '+snippet);
  if(kind==='typescript'&&!containsCodeSnippet(source,'new Map(Object.entries(input))'))throw Error('Retained reference must preserve extras');
  return;
 }
 if(row.id==='native_schema_json_object_strip'){
  if(row.iterations!==2000||row.operationsPerIteration!==2||row.inputText!=='{"count":2,"extra":true}'||row.result!=='int')throw Error('Strip schema denominator');
  for(const snippet of [kind==='native'?'s.object({count:s.int().min(2)}).strip()':'objectSchema()', 'while(i<n){','schema.safeParse(JSON.parse(input))',`schema.safeParse(JSON.parse('{"extra":true}'))`,'error.truncated','error.issues.length!==1','issue.pathTruncated','issue.path.length!==1','segment.index!==undefined',"issue.code==='missing-field'","segment.field!=='count'","issue.expected!=='integer'","issue.observed!=='missing'",kind==='native'?'checksum+=value.count;':'checksum+=success.value.count;','checksum+=1;','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Strip schema operation: '+snippet);
  if(kind==='typescript'&&containsCodeSnippet(source,'Object.keys(input)'))throw Error('Strip reference must discard extras');
  return;
 }
 if(row.id==='native_schema_json_object'){
  if(row.iterations!==2000||row.operationsPerIteration!==2||row.inputText!=='{"count":2}'||row.result!=='int')throw Error('Object schema denominator');
  for(const snippet of [kind==='native'?'s.object({count:s.int().min(2)})':'objectSchema()', 'while(i<n){','schema.safeParse(JSON.parse(input))',`schema.safeParse(JSON.parse('{"extra":true}'))`,'error.truncated','error.issues.length!==2','issue.pathTruncated','issue.path.length!==1','segment.index!==undefined',"issue.code==='missing-field'","segment.field!=='count'","issue.expected!=='integer'","issue.observed!=='missing'","issue.code==='unknown-field'","segment.field!=='extra'","issue.expected!=='missing'","issue.observed!=='boolean'",kind==='native'?'checksum+=value.count;':'checksum+=success.value.count;','checksum+=1;','i++;','return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('Object schema operation: '+snippet);
  return;
 }
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
