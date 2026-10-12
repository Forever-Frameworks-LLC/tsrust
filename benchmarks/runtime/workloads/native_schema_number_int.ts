type Issue = {code:string; expected:string; observed:string; path:({field?:string;index?:number})[]; pathTruncated:boolean};
type Validation = {ok:true;value:number}|{ok:false;error:{issues:Issue[];truncated:boolean}};
// BigInt is the exact integer reference; no JSON parser or unsafe-number input.
const numberSchema = {
  safeParse(input:unknown):Validation {
    if(typeof input!=='bigint')throw 'Reference expects exact integer input';
    const value=Number(input);
    if(BigInt(value)===input)return {ok:true,value};
    return {ok:false,error:{issues:[{code:'number-precision',expected:'number',observed:'integer',path:[],pathTruncated:false}],truncated:false}};
  }
};
function workload(n:number,seed:number,input:string):number {
  const schema=numberSchema;
  let checksum=seed;
  let i=0;
  while(i<n){
    const success=schema.safeParse(42n);
    if(!success.ok)throw 'Expected number success';
    if(success.value!==42)throw 'Expected exact integer';
    checksum+=1;
    const failure=schema.safeParse(9007199254740993n);
    if(failure.ok)throw 'Expected precision rejection';
    const error=failure.error;
    if(error.truncated||error.issues.length!==1)throw 'Invalid error count';
    for(const issue of error.issues){
      if(issue.code!=='number-precision'||issue.expected!=='number'||issue.observed!=='integer'||issue.path.length!==0||issue.pathTruncated)throw 'Invalid issue';
      checksum+=1;
    }
    i++;
  }
  return checksum;
}
