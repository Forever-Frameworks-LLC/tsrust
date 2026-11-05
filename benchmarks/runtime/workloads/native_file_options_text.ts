declare function require(name:string):{
 writeFileSync(path:string|number,bytes:string|Uint8Array):void;appendFileSync(path:string,bytes:Uint8Array):void;
 openSync(path:string,flags:string):number;fstatSync(file:number,options:{bigint:true}):{size:bigint};
 ftruncateSync(file:number,length:number):void;closeSync(file:number):void;readFileSync(path:string):Uint8Array;
 statSync(path:string,options:{bigint:true}):{size:bigint};
};
const fileSystem=require("node:fs");
function workload(n:number,seed:number,input:string):number[][]{
 const destination=input+".out";const initial="hé🦀";const newline=new TextEncoder().encode("\n");const content=new TextEncoder().encode(initial);
 const decoder=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true});const rows:number[][]=[];let i=0;
 while(i<n){fileSystem.writeFileSync(destination,initial);fileSystem.appendFileSync(destination,newline);
 const file=fileSystem.openSync(destination,"r+");let before=0;
 try{before=Number(fileSystem.fstatSync(file,{bigint:true}).size);fileSystem.ftruncateSync(file,0);fileSystem.writeFileSync(file,content);}finally{fileSystem.closeSync(file);}
 const observed=decoder.decode(fileSystem.readFileSync(destination));if(observed!==initial){throw Error("text changed");}
 rows.push([before,Number(fileSystem.statSync(destination,{bigint:true}).size),(i+seed)%17]);i++;}return rows;
}
function main(){}
