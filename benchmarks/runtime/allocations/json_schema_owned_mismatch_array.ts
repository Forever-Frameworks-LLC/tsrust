function workload(n:number,seed:number,input:string):string[]{
 const values:unknown=JSON.parse(input);
 if(!Array.isArray(values))throw Error('shape-mismatch');
 for(const value of values)if(typeof value!=='string')throw Error('shape-mismatch');
 return values as string[];
}
