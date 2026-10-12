type Issue = {code:string; expected:string; observed:string; path:({field?:string;index?:number})[]; pathTruncated:boolean};
type Validation = {ok:true;value:string}|{ok:false;error:{issues:Issue[];truncated:boolean}};
// Matched explicit contract; this is not a Zod comparison.
const stringSchema = {
  safeParse(input:unknown):Validation {
    if(typeof input==='string')return {ok:true,value:input};
    const observed=input===null?'null':Array.isArray(input)?'array':typeof input;
    return {ok:false,error:{issues:[{code:'type-mismatch',expected:'string',observed,path:[],pathTruncated:false}],truncated:false}};
  }
};
function workload(n:number,seed:number,input:string):number {
  const schema=stringSchema;
  let checksum=seed;
  let i=0;
  while(i<n){
    const success=schema.safeParse(JSON.parse(input));
    if(!success.ok)throw 'Expected string success';
    checksum+=success.value.length;
    const failure=schema.safeParse(JSON.parse('42'));
    if(failure.ok)throw 'Expected number rejection';
    const error=failure.error;
    if(error.truncated||error.issues.length!==1)throw 'Invalid error count';
    for(const issue of error.issues){
      if(issue.code!=='type-mismatch'||issue.expected!=='string'||issue.observed!=='number'||issue.path.length!==0||issue.pathTruncated)throw 'Invalid issue';
      checksum+=1;
    }
    i++;
  }
  return checksum;
}
