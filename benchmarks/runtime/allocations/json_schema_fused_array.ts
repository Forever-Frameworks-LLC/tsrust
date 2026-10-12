function workload(n:number,seed:number,input:string):number[]{
 const values:unknown=JSON.parse(input);
 if(!Array.isArray(values))throw Error('shape-mismatch');
 for(const value of values)if(typeof value!=='number'||!Number.isSafeInteger(value))throw Error('integer-range');
 return values as number[];
}
