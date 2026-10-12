type Issue = {code:string; expected:string; observed:string; path:({field?:string;index?:number})[]; pathTruncated:boolean};
type Validation = {ok:true;value:number}|{ok:false;error:{issues:Issue[];truncated:boolean}};
// Matched small-number domain: 7 succeeds and 1.5 fails. Not a full-range JSON parser or Zod comparison.
const integerSchema = {
  safeParse(input:unknown):Validation {
    if(typeof input==='number'&&Number.isInteger(input)&&Number.isSafeInteger(input)&&input>=2&&input<=9)return {ok:true,value:input};
    const observed=input===null?'null':Array.isArray(input)?'array':typeof input;
    return {ok:false,error:{issues:[{code:typeof input!=='number'?'type-mismatch':!Number.isInteger(input)?'integer-fractional':input<2?'too-small':'too-large',expected:'integer',observed,path:[],pathTruncated:false}],truncated:false}};
  }
};
function workload(n:number,seed:number,input:string):number {
  const schema=integerSchema;
  let checksum=seed;
  let i=0;
  while(i<n){
    const success=schema.safeParse(JSON.parse(input));
    if(!success.ok)throw 'Expected integer success';
    checksum+=success.value;
    const failure=schema.safeParse(JSON.parse('10'));
    if(failure.ok)throw 'Expected upper-bound rejection';
    const error=failure.error;
    if(error.truncated||error.issues.length!==1)throw 'Invalid error count';
    for(const issue of error.issues){
      if(issue.code!=='too-large'||issue.expected!=='integer'||issue.observed!=='number'||issue.path.length!==0||issue.pathTruncated)throw 'Invalid issue';
      checksum+=1;
    }
    i++;
  }
  return checksum;
}
