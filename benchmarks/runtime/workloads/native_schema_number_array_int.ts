type Issue={code:string;expected:string;observed:string;path:{index?:number;field?:string}[];pathTruncated:boolean};
type Validation={ok:true;value:number[]}|{ok:false;error:{issues:Issue[];truncated:boolean}};
const numberArraySchema={safeParse(input:bigint[]):Validation{
 const issues:Issue[]=[];let truncated=false;let values:number[]=[];let valid=true;
 for(let index=0;index<input.length;index++){
  const value=Number(input[index]);
  if(BigInt(value)!==input[index]){valid=false;values=[];if(issues.length<64)issues.push({code:'number-precision',expected:'number',observed:'integer',path:[{index}],pathTruncated:false});else truncated=true;}
  else if(valid)values.push(value);
 }
 return valid?{ok:true,value:values}:{ok:false,error:{issues,truncated}};
}};
function workload(n:number,seed:number,input:string):number {
 const schema=numberArraySchema;let checksum=seed;let i=0;
 while(i<n){
  const exact:bigint[]=[42n];
  const success=schema.safeParse(exact);if(!success.ok)throw 'Expected success';
  for(const value of success.value){if(value!==42)throw 'Expected exact conversion';checksum+=1;}
  const lossy:bigint[]=[9007199254740993n];
  const failure=schema.safeParse(lossy);if(failure.ok)throw 'Expected rejection';const error=failure.error;
  if(error.truncated||error.issues.length!==1)throw 'Invalid count';
  for(const issue of error.issues){
   if(issue.code!=='number-precision'||issue.expected!=='number'||issue.observed!=='integer'||issue.path.length!==1||issue.pathTruncated)throw 'Invalid issue';
   for(const segment of issue.path){if(segment.index!==0||segment.field!==undefined)throw 'Invalid path';}
   checksum+=1;
  }
  i++;
 }
 return checksum;
}
