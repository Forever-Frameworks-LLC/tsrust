declare function require(name:string):{readdirSync(path:string,options:{withFileTypes:true}):{name:string;isFile():boolean;isDirectory():boolean;isSymbolicLink():boolean}[]};
const fileSystem=require('node:fs');
interface Entry {name:string;path:string;isFile:boolean;isDirectory:boolean}
function walk(input:string):Entry[]{
 const entries:Entry[]=[];const pending=[input];
 while(pending.length){const directory=pending.pop()!;
  for(const entry of fileSystem.readdirSync(directory,{withFileTypes:true})){
   if(entry.isSymbolicLink())continue;
   if(entry.isDirectory()&&['.git','node_modules','target'].includes(entry.name))continue;
   if(!entry.isFile()&&!entry.isDirectory())continue;
   const path=directory+'/'+entry.name;
   entries.push({name:entry.name,path,isFile:entry.isFile(),isDirectory:entry.isDirectory()});
   if(entry.isDirectory())pending.push(path);
  }
 }
 entries.sort((a,b)=>a.path<b.path?-1:a.path>b.path?1:0);return entries;
}
function workload(n:number,seed:number,input:string):number[][]{
 const rows:number[][]=[];
 for(let i=0;i<n;i++){
  const entries=walk(input);let files=0;let directories=0;let observed=(i+seed)%17;
  for(const entry of entries){if(entry.isFile)files+=1;if(entry.isDirectory)directories+=1;if(entry.name==='é-🦀.txt')observed+=7;if(entry.name==='.hidden')observed+=1;}
  rows.push([entries.length,files,directories,observed]);
 }
 return rows;
}
function main(){}
