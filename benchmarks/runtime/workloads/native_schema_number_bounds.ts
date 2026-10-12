type Issue = {code:string; expected:string; observed:string; path:({field?:string;index?:number})[]; pathTruncated:boolean};
type Validation = {ok:true;value:number}|{ok:false;error:{issues:Issue[];truncated:boolean}};
// Matched explicit contract; this is not a Zod comparison.
const numberSchema = {
  safeParse(input:unknown):Validation {
    if(typeof input==='number'&&Number.isFinite(input)&&input>1&&input<2)return {ok:true,value:input};
    const observed=input===null?'null':Array.isArray(input)?'array':typeof input;
    return {ok:false,error:{issues:[{code:typeof input!=='number'?'type-mismatch':!Number.isFinite(input)?'number-nonfinite':input<=1?'too-small':'too-large',expected:'number',observed,path:[],pathTruncated:false}],truncated:false}};
  }
};
function workload(n:number,seed:number,input:string):number {
  const schema=numberSchema;
  let checksum=seed;
  let i=0;
  while(i<n){
    const success=schema.safeParse(JSON.parse(input));
    if(!success.ok)throw 'Expected number success';
    if(success.value!==1.5)throw 'Expected fraction';
    checksum+=1;
    const failure=schema.safeParse(JSON.parse('2.0'));
    if(failure.ok)throw 'Expected upper-bound rejection';
    const error=failure.error;
    if(error.truncated||error.issues.length!==1)throw 'Invalid error count';
    for(const issue of error.issues){
      if(issue.code!=='too-large'||issue.expected!=='number'||issue.observed!=='number'||issue.path.length!==0||issue.pathTruncated)throw 'Invalid issue';
      checksum+=1;
    }
    i++;
  }
  return checksum;
}
