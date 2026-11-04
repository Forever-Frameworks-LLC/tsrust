// Accept only explicit compiler layout contracts paired with their declaration.
export function fixedArrayResults(emitted) {
 const results=new Map();
 for(const line of emitted.split('\n')) {
  const prefix='// tsrust-fixed-array-result ';
  if(!line.startsWith(prefix))continue;
  let contract;try{contract=JSON.parse(line.slice(prefix.length));}catch{throw Error('Malformed fixed-array result contract');}
  if(!contract||Object.keys(contract).sort().join(',')!=='element,function,rank,result,width'
   ||typeof contract.function!=='string'||!/^[_a-zA-Z][_a-zA-Z0-9]*$/.test(contract.function)
   ||!['i64','f64','bool','String'].includes(contract.element)
   ||![1,2].includes(contract.rank)||!Number.isSafeInteger(contract.width)||contract.width<0
   ||results.has(contract.function))throw Error('Invalid fixed-array result contract');
  const row=`[${contract.element}; ${contract.width}]`;
  const result=contract.rank===2?`Vec<${row}>`:row;
  const escaped=result.replace(/[.*+?^${}()|[\]\\]/g,'\\$&');
  if(contract.result!==result||!new RegExp(`\\bfn ${contract.function}(?:<[^\\n]*>)?\\([^\\n]*\\) -> ${escaped}(?:\\s|\\{)`).test(emitted))throw Error('Fixed-array result declaration disagrees with its contract');
  results.set(contract.function,contract);
 }
 return results;
}

export function fixedArrayResult(contract, sourceResult) {
 const match=/^(int|number|boolean|string)((?:\[\]){1,2})$/.exec(sourceResult);
 if(!match||!contract)return undefined;
 const native={int:'i64',number:'f64',boolean:'bool',string:'String'}[match[1]];
 return contract.element===native&&contract.rank===match[2].length/2?contract.result:undefined;
}
