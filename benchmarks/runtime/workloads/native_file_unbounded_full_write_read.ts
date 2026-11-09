declare function require(name:string):{
 writeFileSync(path:string,bytes:Uint8Array):void;
 readFileSync(path:string):Uint8Array;
};
const fileSystem=require("node:fs");
function sample(input:string,count:number,fill:number):number[]{
 const destination=input+".out";
 {const owner=new Uint8Array(count).fill(fill);fileSystem.writeFileSync(destination,owner);}
 const bytes=fileSystem.readFileSync(destination);
 let sum=0;let index=0;
 while(index<bytes.length){sum+=bytes[index];index++;}
 return [bytes.length,sum];
}
function workload(n:number,seed:number,input:string):number[][]{
 const output:number[][]=[];let i=0;
 while(i<n){let count=65536;if(i%3===1){count=1048576;}if(i%3===2){count=67108865;}
 const fill=(i+seed)%256;output.push(sample(input,count,fill));i++;}
 return output;
}
function main(){}
