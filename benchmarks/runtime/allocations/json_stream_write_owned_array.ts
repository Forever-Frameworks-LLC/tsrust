declare function require(name:string):{
 openSync(path:string,flags:string):number;
 writeSync(fd:number,buffer:Uint8Array,offset:number,length:number,position:null):number;
 closeSync(fd:number):void;
};
const fileSystem=require('node:fs');
function workload(n:number,seed:number,input:string):number{
 const file=fileSystem.openSync(input,'w');
 const buffer=new Uint8Array(8192);let used=0;
 const encoder=new TextEncoder();
 const item={active:true,name:'hello',values:[20,17]};
 function flush():void{
  let offset=0;
  while(offset<used){const count=fileSystem.writeSync(file,buffer,offset,used-offset,null);if(count===0)throw Error('write-zero');offset+=count;}
  used=0;
 }
 try{
  for(let i=0;i<n;i++){
   const bytes=encoder.encode(JSON.stringify(item)+'\n');
   if(bytes.length-1>16777216)throw Error('output-limit');
   for(const byte of bytes){if(used===buffer.length)flush();buffer[used++]=byte;}
  }
  flush();
 }finally{fileSystem.closeSync(file);}
 return seed+n;
}
