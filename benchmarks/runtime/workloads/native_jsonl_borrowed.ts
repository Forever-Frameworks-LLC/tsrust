declare function require(name: string): {
 openSync(path:string,flags:string):number;
 readSync(fd:number,buffer:Uint8Array,offset:number,length:number,position:null):number;
 closeSync(fd:number):void;
};
const fileSystem=require('node:fs');
function observe(line:string):number{
 const value:unknown=JSON.parse(line);
 if(value===null||typeof value!=='object'||Array.isArray(value))throw Error('shape-mismatch');
 const item=value as Record<string,unknown>;
 if(typeof item.name!=='string'||typeof item.active!=='boolean'||!Array.isArray(item.values))throw Error('shape-mismatch');
 for(const number of item.values)if(typeof number!=='number'||!Number.isSafeInteger(number))throw Error('integer-range');
 if(!item.active)throw Error('Inactive item');
 return item.name.length+(item.values[0] as number)+(item.values[1] as number);
}
function workload(n:number,seed:number,input:string):number{
 let checksum=seed;let i=0;
 while(i<n){
  const file=fileSystem.openSync(input,'r');
  try{
   const buffer=new Uint8Array(8192);const decoder=new TextDecoder('utf-8',{fatal:true});let pending='';
   while(true){
    const count=fileSystem.readSync(file,buffer,0,buffer.length,null);
    pending+=decoder.decode(buffer.subarray(0,count),{stream:count!==0});
    let newline=pending.indexOf('\n');
    while(newline>=0){checksum+=observe(pending.slice(0,newline));pending=pending.slice(newline+1);newline=pending.indexOf('\n');}
    if(pending.length>16777216)throw Error('byte-limit');
    if(count===0){if(pending.length!==0)checksum+=observe(pending);break;}
   }
  }finally{fileSystem.closeSync(file);}
  i++;
 }
 return checksum;
}
