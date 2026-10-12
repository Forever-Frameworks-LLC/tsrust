function workload(n:number,seed:number,input:string):number {
 let checksum=seed;let i=0;
 while(i<n){
  const values=[42,null];
  const entries:[string,unknown][]=[['items',values],['name','old'],['name',input]];
  const object=Object.fromEntries(entries);checksum+=JSON.stringify(object).length;
  i++;
 }
 return checksum;
}
