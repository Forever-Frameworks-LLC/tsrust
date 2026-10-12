type Issue={code:string;expected:string;observed:string;path:{index?:number;field?:string}[];pathTruncated:boolean};
type Validation={ok:true;value:{count:number}}|{ok:false;error:{issues:Issue[];truncated:boolean}};
function objectSchema(){return {safeParse(input:Record<string,unknown>):Validation{
 const issues:Issue[]=[];const count=input.count;delete input.count;
 if(count===undefined)issues.push({code:'missing-field',expected:'integer',observed:'missing',path:[{field:'count'}],pathTruncated:false});
 else if(typeof count!=='number'||!Number.isInteger(count)||count<2)throw 'Invalid benchmark input';
 for(const key of Object.keys(input))issues.push({code:'unknown-field',expected:'missing',observed:typeof input[key],path:[{field:key}],pathTruncated:false});
 return issues.length===0?{ok:true,value:{count:count as number}}:{ok:false,error:{issues,truncated:false}};
}};}
function workload(n:number,seed:number,input:string):number {
 const schema=objectSchema();let checksum=seed;let i=0;
 while(i<n){
  const success=schema.safeParse(JSON.parse(input));if(!success.ok)throw 'Expected success';checksum+=success.value.count;
  const failure=schema.safeParse(JSON.parse('{"extra":true}'));if(failure.ok)throw 'Expected rejection';const error=failure.error;
  if(error.truncated||error.issues.length!==2)throw 'Invalid count';
  for(const issue of error.issues){
   if(issue.pathTruncated||issue.path.length!==1)throw 'Invalid path';
   for(const segment of issue.path){
    if(segment.index!==undefined)throw 'Unexpected index';
    if(issue.code==='missing-field'){if(segment.field!=='count'||issue.expected!=='integer'||issue.observed!=='missing')throw 'Invalid missing field';}
    else if(issue.code==='unknown-field'){if(segment.field!=='extra'||issue.expected!=='missing'||issue.observed!=='boolean')throw 'Invalid unknown field';}
    else throw 'Unexpected issue';
   }
   checksum+=1;
  }
  i++;
 }
 return checksum;
}
