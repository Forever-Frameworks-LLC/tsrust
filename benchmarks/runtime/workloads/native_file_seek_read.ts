declare function require(name:string):{
 openSync(path:string,flags:string):number;closeSync(file:number):void;
 readSync(file:number,bytes:Uint8Array,offset:number,length:number,position:number):number;
};
const fileSystem=require("node:fs");
function workload(n:number,seed:number,input:string):number[][] {
 const file=fileSystem.openSync(input,"r");const rows:number[][]=[];let i=0;let position=0;
 try {
  while(i<n){position=i%8===0?4096:(i*97+seed)%4096;const bytes=new Uint8Array(64);const count=fileSystem.readSync(file,bytes,0,64,position);position+=count;
   let sum=0;let index=0;while(index<count){sum+=bytes[index];index++;}rows.push([position,sum]);i++;}
 }finally{fileSystem.closeSync(file);}
 return rows;
}
function main(){}
