type Issue={code:string;expected:string;observed:string;path:never[];pathTruncated:boolean};
type Validation={ok:true;value:'ready'|'done'}|{ok:false;error:{issues:Issue[];truncated:boolean}};
function enumSchema(){return {safeParse(input:unknown):Validation{
 if(input==='ready'||input==='done')return {ok:true,value:input};
 return {ok:false,error:{issues:[{code:typeof input==='string'?'literal-mismatch':'type-mismatch',expected:'string',observed:input===null?'null':typeof input,path:[],pathTruncated:false}],truncated:false}};
}};}
function workload(n:number,seed:number,input:string):number{
 const schema=enumSchema();let checksum=seed;let i=0;
 while(i<n){
  const success=schema.safeParse(JSON.parse(input));if(!success.ok)throw 'Expected success';const value=success.value;
  if(value!=='ready')throw 'Invalid enum value';checksum+=value.length;
  const failure=schema.safeParse(JSON.parse('"waiting"'));if(failure.ok)throw 'Expected rejection';const error=failure.error;
  if(error.truncated||error.issues.length!==1)throw 'Invalid issue count';
  for(const issue of error.issues){if(issue.code!=='literal-mismatch'||issue.expected!=='string'||issue.observed!=='string'||issue.pathTruncated||issue.path.length!==0)throw 'Invalid issue';}
  checksum+=1;i++;
 }
 return checksum;
}
