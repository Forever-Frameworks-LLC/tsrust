declare function require(name:string):{
 openSync(path:string,flags:string):number;
 writeSync(fd:number,buffer:Uint8Array,offset:number,length:number,position:null):number;
 closeSync(fd:number):void;
};
const fileSystem=require('node:fs');
type Item={active:boolean;name:string;values:number[];};
function workload(n:number,seed:number,input:string):number{
 let checksum=seed;let i=0;
 while(i<n){
  const item:Item={active:true,name:'hello',values:[20,17]};
  const file=fileSystem.openSync(input,'w');
  try{
   const bytes=new TextEncoder().encode(JSON.stringify(item)+'\n');
   if(bytes.length-1>16777216)throw Error('output-limit');
   let offset=0;
   while(offset<bytes.length){const count=fileSystem.writeSync(file,bytes,offset,bytes.length-offset,null);if(count===0)throw Error('write-zero');offset+=count;}
  }finally{fileSystem.closeSync(file);}
  checksum+=1;i++;
 }
 return checksum;
}
