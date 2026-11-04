type Issue = {code:string;expected:string;observed:string;path:{field?:string;index?:number}[];pathTruncated:boolean};
type Failure = {issues:Issue[];truncated:boolean};
type Outcome = {success:true;data:{value:{count:number};extra:Map<string,unknown>}} | {success:false;error:{kind:'validation';error:Failure}};
function objectSchema() {
  return {safeParseJson(text:string):Outcome {
    const input:Record<string,unknown> = JSON.parse(text);
    if (!Object.hasOwn(input,'count')) {
      return {success:false,error:{kind:'validation',error:{issues:[{code:'missing-field',expected:'integer',observed:'missing',path:[{field:'count'}],pathTruncated:false}],truncated:false}}};
    }
    const count=input.count;
    if(typeof count!=='number'||!Number.isInteger(count)||count<2)throw 'Invalid benchmark input';
    delete input.count;
    return {success:true,data:{value:{count},extra:new Map(Object.entries(input))}};
  }};
}
function workload(n:number,seed:number,input:string):number {
  const schema=objectSchema();
  let checksum=seed;
  let i=0;
  while(i<n){
    {
      const outcome=schema.safeParseJson(input);
      if(outcome.success){
        const value=outcome.data;
        const extra=value.extra.get('extra');
        if(extra===undefined||value.extra.size!==1)throw 'Invalid extras';
        if(JSON.stringify(extra)!=='{"tag":"ready","values":[1,2,3]}')throw 'Retained payload changed';
        checksum+=value.value.count+value.extra.size;
      }else{throw 'Expected source success';}
    }
    {
      const outcome=schema.safeParseJson('{"extra":true}');
      if(outcome.success){throw 'Expected source rejection';}
      else{
        const failure=outcome.error;
        if(failure.kind==='validation'){
          const error=failure.error;
          if(error.truncated||error.issues.length!==1)throw 'Invalid count';
          for(const issue of error.issues){
            if(issue.code!=='missing-field'||issue.expected!=='integer'||issue.observed!=='missing'||issue.pathTruncated||issue.path.length!==1||issue.path[0].field!=='count'||issue.path[0].index!==undefined)throw 'Invalid missing-field issue';
            checksum+=1;
          }
        }else{throw 'Expected validation error';}
      }
    }
    i++;
  }
  return checksum;
}
