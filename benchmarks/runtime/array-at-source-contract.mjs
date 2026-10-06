export function assertArrayAtSource(source,row,kind){
 const fragments=['function workload(n: int, seed: int','const count = (i + seed) % 4;','const index = (i * 5 + seed) % 9 - 4;','const selected = values.at(index);','output.push(typeof selected);','output.push(String(values.length));','return output;'];
 if(row.atElement!=='undefined')fragments.push('if (selected === undefined)','output.push(describe(selected))');
 for(let j=0;j<3;j++)fragments.push('if (values.length > '+j+') { output.push(describe(values['+j+'])); } else { output.push("<absent>"); }');
 if(row.atElement==='nested')fragments.push('String(value.length).concat(":", value.join(","))');if(row.atElement==='record')fragments.push('String(value.id).concat(":", value.label)');
 for(const fragment of fragments)if(!source.includes(fragment))throw Error('Missing actual lookup/observation: '+fragment);
 if((source.match(/values\.at\(index\)/g)??[]).length!==1||source.includes('clone(')||/values\.(pop|shift|splice)\(/.test(source))throw Error('One borrowed lookup, receiver unchanged');
 if(kind==='typescript'&&!source.startsWith('type int = number;'))throw Error('TS pair alias');
}
