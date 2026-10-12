function workload(n:number,seed:number,input:string):number {
 let checksum=seed;let i=0;
 while(i<n){
  checksum+=JSON.stringify(null).length;
  checksum+=JSON.stringify(true).length;
  checksum+=JSON.stringify(42).length;
  const value=1.25;if(!Number.isFinite(value)){throw 'number-nonfinite';}
  checksum+=JSON.stringify(value).length;
  checksum+=JSON.stringify(input).length;
  i++;
 }
 return checksum;
}
