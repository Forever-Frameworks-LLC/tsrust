type Issue={code:string;expected:string;observed:string;path:{index?:number;field?:string}[];pathTruncated:boolean};
type Validation={ok:true;value:{count:number;label:string}}|{ok:false;error:{issues:Issue[];truncated:boolean}};
function objectSchema(){const factory=()=>7;return {safeParse(input:Record<string,unknown>):Validation{
 const issues:Issue[]=[];const count=Object.hasOwn(input,'count')?input.count:factory(),label=input.label;
 if(typeof count!=='number'||!Number.isSafeInteger(count))issues.push({code:'type-mismatch',expected:'integer',observed:count===null?'null':typeof count,path:[{field:'count'}],pathTruncated:false});
 else if(count<5)issues.push({code:'too-small',expected:'integer',observed:'number',path:[{field:'count'}],pathTruncated:false});
 if(label===undefined)issues.push({code:'missing-field',expected:'string',observed:'missing',path:[{field:'label'}],pathTruncated:false});
 else if(typeof label!=='string')issues.push({code:'type-mismatch',expected:'string',observed:typeof label,path:[{field:'label'}],pathTruncated:false});
 for(const key of Object.keys(input))if(key!=='count'&&key!=='label')issues.push({code:'unknown-field',expected:'missing',observed:typeof input[key],path:[{field:key}],pathTruncated:false});
 return issues.length===0?{ok:true,value:{count:count as number,label:label as string}}:{ok:false,error:{issues,truncated:false}};
}};}
function workload(n:number,seed:number,input:string):number{
 const schema=objectSchema();let checksum=seed;let i=0;
 while(i<n){
  const success=schema.safeParse(JSON.parse(input));if(!success.ok)throw 'Expected success';const value=success.value;
  if(value.count!==7||value.label!=='ready')throw 'Invalid presence';checksum+=value.count;
  const failure=schema.safeParse(JSON.parse('{"count":null}'));if(failure.ok)throw 'Expected rejection';const error=failure.error;
  if(error.truncated||error.issues.length!==2)throw 'Invalid count';
  for(const issue of error.issues){
   if(issue.pathTruncated||issue.path.length!==1)throw 'Invalid path';
   for(const segment of issue.path){
    if(segment.index!==undefined)throw 'Unexpected index';
    if(segment.field==='count'){if(issue.code!=='type-mismatch'||issue.expected!=='integer'||issue.observed!=='null')throw 'Invalid null issue';}
    else if(segment.field==='label'){if(issue.code!=='missing-field'||issue.expected!=='string'||issue.observed!=='missing')throw 'Invalid missing issue';}
    else throw 'Unexpected field';
   }
   checksum+=1;
  }
  i++;
 }
 return checksum;
}
