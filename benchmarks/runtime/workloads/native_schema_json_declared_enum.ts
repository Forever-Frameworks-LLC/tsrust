enum Code{Low=1,High=2.5}
type Issue={code:string;expected:string;observed:string;path:never[];pathTruncated:boolean};
type Validation={ok:true;value:Code}|{ok:false;error:{issues:Issue[];truncated:boolean}};
function enumSchema(){return {safeParse(input:unknown):Validation{
 if(input===Code.Low||input===Code.High)return {ok:true,value:input};
 return {ok:false,error:{issues:[{code:typeof input==='number'?'literal-mismatch':'type-mismatch',expected:'number',observed:input===null?'null':typeof input,path:[],pathTruncated:false}],truncated:false}};
}};}
function workload(n:number,seed:number,input:string):number{
 const schema=enumSchema();let checksum=seed;let i=0;
 while(i<n){
  const success=schema.safeParse(JSON.parse(input));if(!success.ok)throw 'Expected success';const value=success.value;
  if(value!==Code.High)throw 'Invalid enum value';checksum+=1;
  const failure=schema.safeParse(JSON.parse('3.0'));if(failure.ok)throw 'Expected rejection';const error=failure.error;
  if(error.truncated||error.issues.length!==1)throw 'Invalid issue count';
  for(const issue of error.issues){if(issue.code!=='literal-mismatch'||issue.expected!=='number'||issue.observed!=='number'||issue.pathTruncated||issue.path.length!==0)throw 'Invalid issue';}
  checksum+=1;i++;
 }
 return checksum;
}
