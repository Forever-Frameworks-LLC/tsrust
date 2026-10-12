function workload(n:number,seed:number,input:string):number[]{
 const record:unknown=JSON.parse(input);
 if(typeof record!=='object'||record===null||Array.isArray(record)||Object.keys(record).length!==1||!('items' in record))throw Error('shape-mismatch');
 const values=record.items;
 if(!Array.isArray(values))throw Error('shape-mismatch');
 for(const value of values)if(typeof value!=='number'||!Number.isSafeInteger(value))throw Error('integer-range');
 return values as number[];
}
