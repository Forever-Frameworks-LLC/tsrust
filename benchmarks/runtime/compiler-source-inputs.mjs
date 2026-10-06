// Bundled library surfaces and native backends are compiler inputs, including nested files.
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
export function compilerSourceInputs(directory){
 const inputs=[];
 function visit(current){
  for(const entry of fs.readdirSync(current,{withFileTypes:true})){
   const file=path.join(current,entry.name);
   if(entry.isDirectory())visit(file);
   else if(entry.isFile()&&/\.(rs|tsr)$/.test(entry.name))inputs.push([path.relative(directory,file).split(path.sep).join('/'),createHash('sha256').update(fs.readFileSync(file)).digest('hex')]);
  }
 }
 visit(directory);
 return inputs.sort(([left],[right])=>left.localeCompare(right));
}
