type ValidationError = {issues:{code:string;path:unknown[];pathTruncated:boolean}[];truncated:boolean};
function stringSchema() {
  return {parse(input:string):string {
    if (Array.from(input).length < 1) {
      throw {issues:[{code:'too-small',path:[],pathTruncated:false}],truncated:false} satisfies ValidationError;
    }
    return input;
  }};
}
function workload(n:number,seed:number,input:string):number {
  const schema=stringSchema();
  let checksum=seed;
  let i=0;
  while(i<n){
    const value=schema.parse(input);
    if(value!=='hello'){throw 'Wrong parsed value';}
    checksum+=value.length;
    let rejected=false;
    try{schema.parse('');}
    catch(caught){
      const error=caught as ValidationError;
      if(error.truncated||error.issues.length!==1){throw 'Wrong issue count';}
      for(const issue of error.issues){
        if(issue.code!=='too-small'||issue.path.length!==0||issue.pathTruncated){throw 'Wrong issue';}
      }
      rejected=true;
      checksum+=1;
    }
    if(!rejected){throw 'Expected rejection';}
    i++;
  }
  return checksum;
}
