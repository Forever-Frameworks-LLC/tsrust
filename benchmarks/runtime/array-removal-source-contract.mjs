import {containsCodeSnippet} from './source-contract-tokens.mjs';
export function assertArrayRemovalSource(source,row,kind){
 const expectedMethod=row.removalMethod,other=expectedMethod==='pop'?'shift':'pop';if(!['pop','shift'].includes(expectedMethod))throw Error('Removal method');
 const required=['function workload(n: int, seed: int','const count = (i + seed) % 4;','if (count > 0)','if (count > 1)','if (count > 2)','const removed0 = values.'+expectedMethod+'();','const removed1 = values.'+expectedMethod+'();','output.push(typeof removed0);','output.push(typeof removed1);','return output;'];
 if(row.removalElement!=='undefined')required.push('if (removed0 === undefined)','if (removed1 === undefined)','output.push(describe(removed0))','output.push(describe(removed1))');
 if(row.removalElement==='nested')required.push('String(value.length).concat(":", value.join(","))');if(row.removalElement==='record')required.push('String(value.id).concat(":", value.label)');
 for(const fragment of required)if(!containsCodeSnippet(source, fragment))throw Error('Missing actual removal/observation: '+fragment);
 if((source.match(new RegExp('values\\.'+expectedMethod+'\\(\\)','g'))??[]).length!==2||containsCodeSnippet(source, 'values.'+other+'(')||containsCodeSnippet(source, 'clone('))throw Error('Two real removals without inserted clone');
 for(const fragment of ['output.push(String(values.length));','output.push(describe(values[0]));','output.push(describe(values[1]));'])if(source.split(fragment).length!==3)throw Error('Observe remaining order twice');
 if(kind==='typescript'&&!source.startsWith('type int = number;'))throw Error('TS pair alias');
}
