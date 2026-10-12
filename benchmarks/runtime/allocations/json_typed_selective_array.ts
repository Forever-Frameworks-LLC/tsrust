function workload(n:number,seed:number,input:string):{items:number[][]}{
 const value:unknown=JSON.parse(input);
 if(typeof value!=='object'||value===null||!('items' in value)||!Array.isArray(value.items))throw Error('shape-mismatch');
 for(const row of value.items){
  if(!Array.isArray(row))throw Error('shape-mismatch');
  for(const item of row)if(typeof item!=='number'||!Number.isSafeInteger(item))throw Error('integer-range');
 }
 return {items:value.items as number[][]};
}
