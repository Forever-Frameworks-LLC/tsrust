declare function require(name:string):{
 openSync(path:string,flags:string):number;
 readSync(file:number,buffer:Uint8Array,offset:number,length:number,position:null):number;
 writeFileSync(path:string,bytes:Uint8Array):void;
 closeSync(file:number):void;
};
const fileSystem=require("node:fs");
function sample(input:string,count:number):number[]{
 const destination=input+".out";
 const owner=new TextEncoder().encode("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
 const bytes=owner.subarray(0,count);
 fileSystem.writeFileSync(destination,bytes);
 const file=fileSystem.openSync(destination,"r");
 try {
  const limit=count+1;
  const readback=new Uint8Array(limit+1);
  let size=0;
  while(true){
   const got=fileSystem.readSync(file,readback,size,limit-size+1,null);
   if(got===0){break;}
   size+=got;
   if(size>limit){throw Error("length-range");}
  }
  let sum=0;let index=0;
  while(index<size){sum+=readback[index];index++;}
  return [size,sum];
 }finally{fileSystem.closeSync(file);}
}
function workload(n:number,seed:number,input:string):number[][]{
 const output:number[][]=[];let i=0;
 while(i<n){const count=(i+seed)%64+1;output.push(sample(input,count));i++;}
 return output;
}
function main(){}
