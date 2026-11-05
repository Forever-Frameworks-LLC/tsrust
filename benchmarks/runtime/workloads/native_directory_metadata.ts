declare function require(name:string):{
 readdirSync(path:string,options:{withFileTypes:true}):{name:string;isFile():boolean;isDirectory():boolean;isSymbolicLink():boolean}[];
 statSync(path:string,options:{bigint:true}):{size:bigint;isFile():boolean;isDirectory():boolean};
};
const fileSystem=require("node:fs");
function sample(input:string,tag:number):number[]{
 const entries=fileSystem.readdirSync(input,{withFileTypes:true}).map(entry=>({name:entry.name,path:input+"/"+entry.name,isFile:entry.isFile(),isDirectory:entry.isDirectory(),isSymlink:entry.isSymbolicLink()}));
 let files=0;let bytes=0;let observed=tag;
 for(const entry of entries){const metadata=fileSystem.statSync(entry.path,{bigint:true});
 if(metadata.isFile()!==entry.isFile||metadata.isDirectory()!==entry.isDirectory){throw Error("file kind changed");}
 if(entry.isFile){files++;bytes+=Number(metadata.size);}if(entry.isDirectory){observed++;}if(entry.isSymlink){observed+=16;}
 if(entry.name==="é-🦀.txt"){observed+=7;}if(entry.name===".hidden"){observed++;}}
 return [entries.length,files,bytes,observed];
}
function workload(n:number,seed:number,input:string):number[][]{const rows:number[][]=[];let i=0;while(i<n){rows.push(sample(input,(i+seed)%17));i++;}return rows;}
function main(){}
