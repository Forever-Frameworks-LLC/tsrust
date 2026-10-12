type Item={active:boolean;name:string;values:number[];};
function workload(n:number,seed:number,input:string):number{
 let checksum=seed;let i=0;
 while(i<n){
  const item:Item={active:true,name:input,values:[20,17]};
  const text=JSON.stringify(item);
  if(text!=='{"active":true,"name":"hello","values":[20,17]}'){throw 'Unexpected JSON output';}
  checksum+=text.length;i++;
 }
 return checksum;
}
