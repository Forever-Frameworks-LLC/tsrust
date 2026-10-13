type Issue = {message:string;code:string; expected:string; observed:string; path:({field?:string;index?:number})[]; pathTruncated:boolean};
type Validation = {ok:true;value:string}|{ok:false;error:{issues:Issue[];truncated:boolean}};
// Matched explicit contract; this is not a Zod comparison.
const stringSchema = {
  safeParse(input:unknown):Validation {
    const length=typeof input==='string'?[...input].length:0;
    if(typeof input==='string'&&length>=2&&length<=6)return {ok:true,value:input};
    const observed=input===null?'null':Array.isArray(input)?'array':typeof input;
    return {ok:false,error:{issues:[{message:typeof input!=='string'?'type-mismatch':length<2?'Too short':'Too long',code:typeof input!=='string'?'type-mismatch':length<2?'too-small':'too-large',expected:'string',observed,path:[],pathTruncated:false}],truncated:false}};
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
    const failure=schema.safeParse(JSON.parse('"x"'));
    if(failure.ok)throw 'Expected short string rejection';
    const error=failure.error;
    if(error.truncated||error.issues.length!==1)throw 'Invalid error count';
    for(const issue of error.issues){
      if(issue.message!=='Too short'||issue.code!=='too-small'||issue.expected!=='string'||issue.observed!=='string'||issue.path.length!==0||issue.pathTruncated)throw 'Invalid issue';
      checksum+=1;
    }
    i++;
  }
  return checksum;
}
