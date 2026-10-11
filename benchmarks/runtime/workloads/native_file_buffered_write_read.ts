declare function require(name:string):{
 openSync(path:string,flags:string):number;
 readSync(file:number,buffer:Uint8Array,offset:number,length:number,position:null):number;
 writeSync(file:number,buffer:Uint8Array,offset:number,length:number,position:null):number;
 closeSync(file:number):void;
};
const fileSystem=require("node:fs");
function flush(file:number,buffer:Uint8Array,used:number):void {
 let written=0;
 while(written<used){
  const size=fileSystem.writeSync(file,buffer,written,used-written,null);
  if(size===0){throw Error("write-zero");}
  written+=size;
 }
}
function sample(input:string,count:number):number[]{
 const destination=input+".out";
 const output=fileSystem.openSync(destination,"w");
 try {
  const buffer=new Uint8Array(16);let used=0;
  const owner=new TextEncoder().encode("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
  let offset=0;
  while(offset<count){
   let end=offset+3;if(end>count){end=count;}
   const bytes=owner.subarray(offset,end);
   if(bytes.length>buffer.length-used){flush(output,buffer,used);used=0;}
   buffer.set(bytes,used);used+=bytes.length;offset=end;
  }
  flush(output,buffer,used);
 }finally{fileSystem.closeSync(output);}
 const file=fileSystem.openSync(destination,"r");
 try {
  const buffer=new Uint8Array(16);let start=0;let end=0;
  let total=0;let sum=0;
  while(true){
   if(start===end){end=fileSystem.readSync(file,buffer,0,buffer.length,null);start=0;}
   const size=Math.min(3,end-start);
   const bytes=buffer.slice(start,start+size);start+=size;
   if(size===0){break;}
   total+=size;let index=0;
   while(index<size){sum+=bytes[index];index++;}
  }
  return [total,sum];
 }finally{fileSystem.closeSync(file);}
}
function workload(n:number,seed:number,input:string):number[][]{
 const output:number[][]=[];let i=0;
 while(i<n){const count=(i+seed)%64+1;output.push(sample(input,count));i++;}
 return output;
}
function main(){}
