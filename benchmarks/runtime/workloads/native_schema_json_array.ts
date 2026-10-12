type Issue={code:string;expected:string;observed:string;path:{index?:number;field?:string}[];pathTruncated:boolean};
type Validation={ok:true;value:number[]}|{ok:false;error:{issues:Issue[];truncated:boolean}};
function arraySchema(min:number){return {safeParse(input:number[]):Validation{
 const issues:Issue[]=[];let truncated=false;let value:number[]=[];let valid=true;
 for(let index=0;index<input.length;index++){
  const item=input[index];
  if(item<min){valid=false;value=[];if(issues.length<64)issues.push({code:'too-small',expected:'integer',observed:'number',path:[{index}],pathTruncated:false});else truncated=true;}
  else if(valid)value.push(item);
 }
 return valid?{ok:true,value}:{ok:false,error:{issues,truncated}};
}};}
function workload(n:number,seed:number,input:string):number {
 const schema=arraySchema(2);let checksum=seed;let i=0;
 while(i<n){
  const success=schema.safeParse(JSON.parse(input));if(!success.ok)throw 'Expected success';checksum+=success.value.length;
  const failure=schema.safeParse(JSON.parse('[1,0]'));if(failure.ok)throw 'Expected rejection';const error=failure.error;
  if(error.truncated||error.issues.length!==2)throw 'Invalid count';
  let index=0;
  for(const issue of error.issues){
   if(issue.code!=='too-small'||issue.expected!=='integer'||issue.observed!=='number'||issue.path.length!==1||issue.pathTruncated)throw 'Invalid issue';
   for(const segment of issue.path){if(segment.index!==index||segment.field!==undefined)throw 'Invalid path';}
   checksum+=1;index++;
  }
  i++;
 }
 return checksum;
}
