function workload(n:number,seed:number,input:string):number[][]{
 const values:unknown=JSON.parse(input);
 if(!Array.isArray(values))throw Error('shape-mismatch');
 for(const row of values){
  if(!Array.isArray(row))throw Error('shape-mismatch');
  for(const value of row)if(typeof value!=='number'||!Number.isSafeInteger(value))throw Error('integer-range');
 }
 return values as number[][];
}
