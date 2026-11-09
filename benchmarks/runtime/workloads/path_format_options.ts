declare function require(name:string): {parse(value:string):{root:string;dir:string;base:string;ext:string;name:string};format(parts:{root?:string;dir?:string;base?:string;ext?:string;name?:string}):string;resolve(...paths:string[]):string;sep:string};
const path=require('node:path');

function workload(n:number,seed:number,input:string):number{const parts={dir:String(input),name:'file.tar',ext:'gz'};let checksum=seed;let i=0;while(i<n){const value=path.format(parts);if(value.length!==19||!value.startsWith('reports')||!value.endsWith('file.tar.gz')){throw 'path format';}checksum+=value.length;i++;}return checksum;}
