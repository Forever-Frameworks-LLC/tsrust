function workload(n:number,seed:number,input:string):number {
 const owner:unknown=JSON.parse(input);
 let checksum=seed;let i=0;
 while(i<n){
  if(typeof owner!=='string'){throw 'expected-string';}
  const text=owner;checksum+=text.length+text.indexOf('hello')+text.lastIndexOf('hello');
  i++;
 }
 return checksum;
}
