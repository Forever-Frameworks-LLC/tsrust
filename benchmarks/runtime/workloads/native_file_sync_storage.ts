declare function require(name:string):{
 openSync(path:string,flags:string):number;closeSync(file:number):void;
 writeSync(file:number,bytes:Uint8Array,offset:number,length:number,position:null):number;
 readSync(file:number,bytes:Uint8Array,offset:number,length:number,position:number):number;
 fdatasyncSync(file:number):void;fsyncSync(file:number):void;
};
const fileSystem=require("node:fs");
function workload(n:number,seed:number,input:string):number[][] {
 const destination=input+".out";const file=fileSystem.openSync(destination,"w+");const first=new TextEncoder().encode("a");const second=new TextEncoder().encode("b");const rows:number[][]=[];let i=0;let position=0;
 try {
  while(i<n){if(fileSystem.writeSync(file,first,0,1,null)!==1){throw Error("short sync write");}fileSystem.fdatasyncSync(file);
   if(fileSystem.writeSync(file,second,0,1,null)!==1){throw Error("short sync write");}fileSystem.fsyncSync(file);
   const bytes=new Uint8Array(2);const count=fileSystem.readSync(file,bytes,0,2,position);if(count!==2){throw Error("short sync readback");}position+=count;
   rows.push([position,bytes[0]+bytes[1]+(i+seed)%17]);i++;}
 }finally{fileSystem.closeSync(file);}
 return rows;
}
function main(){}
