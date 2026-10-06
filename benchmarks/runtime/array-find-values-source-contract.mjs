export function assertFindValueSource(source,row,kind){
 const typ={nested:'int[]',record:'FindRecord'}[row.findElement]??row.findElement;
 const required=['function workload(n: int, seed: int','const count = (i + seed) % 4;','const mode = (i * 7 + seed) % 5;',`const selected = values.${row.findMethod}((value: ${typ}, index: int, source: ${typ}[]): boolean => {`,'visits = visits + 1;','trace = trace * 10 + index + 1;','sourceVisits = sourceVisits + source.length;','return source.length === count &&','mode === 1 && index === 0 || mode === 2 && index % 2 === 0 || mode === 3 && index >= 1 || mode === 4','return output;'];
 if(row.findElement==='number')required.push('if (selected === undefined) { output.push(0.0); output.push(0.0 / 0.0); } else { output.push(1.0); output.push(selected); }','output.push(Number(values.length)); output.push(Number(visits)); output.push(Number(trace)); output.push(Number(sourceVisits));');
 else{required.push('output.push(typeof selected);','output.push(String(values.length)); output.push(String(visits)); output.push(String(trace)); output.push(String(sourceVisits));');if(row.findElement!=='undefined')required.push('if (selected === undefined)','output.push(describe(selected))');}
 const observation=row.findElement==='number'?'values':'describe(values';
 for(let j=0;j<3;j++)required.push(`if (values.length > ${j}) { output.push(${observation}[${j}]${row.findElement==='number'?'':')'}); } else { output.push(${row.findElement==='number'?'0.0 / 0.0':'"<absent>"'}); }`);
 for(const text of required)if(!source.includes(text))throw Error('Missing actual find/traversal/whole observation: '+text);
 if((source.match(/values\.(find|findLast)\(/g)??[]).length!==1||source.includes('clone(')||/values\.(pop|shift|splice)\(/.test(source))throw Error('One readonly selected lookup');
 if(kind==='typescript'&&!source.startsWith('type int = number;'))throw Error('TS pair alias');
}
