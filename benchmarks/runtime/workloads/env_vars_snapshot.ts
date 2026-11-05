declare const Buffer:{byteLength(value:string,encoding:string):number};
declare function require(name:string):{env:Record<string,string|undefined>};
const environment=require('node:process').env;

function workload(n:number,seed:number):number{let checksum=seed;let i=0;while(i<n){const rows=Object.entries(environment).map(([name,value])=>({name,value:value??''}));for(const row of rows){if(row.name==='TSRUST_B03_TEXT'||row.name==='TSRUST_B03_EMPTY'){checksum+=row.name.length+Buffer.byteLength(row.value,'utf8');}}i++;}return checksum;}
