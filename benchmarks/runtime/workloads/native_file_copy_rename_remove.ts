declare function require(name:string):{
 copyFileSync(from:string,to:string):void;renameSync(from:string,to:string):void;unlinkSync(path:string):void;
 statSync(path:string,options:{bigint:true}):{size:bigint};
};
const fileSystem=require("node:fs");
function workload(n:number,seed:number,input:string):number[][]{
 const source=input+"/source.bin";const target=input+"/copy.bin";const moved=input+"/moved.bin";const rows:number[][]=[];
 let i=0;while(i<n){fileSystem.copyFileSync(source,target);const length=Number(fileSystem.statSync(target,{bigint:true}).size);
 fileSystem.renameSync(target,moved);fileSystem.unlinkSync(moved);rows.push([length,length+(i+seed)%17]);i++;}return rows;
}
function main(){}
