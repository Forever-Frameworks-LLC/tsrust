// Node24 accepts exact decimal strings; this assertion only bridges the TS Intl declaration.
function workload(n:number,seed:number,input:string):number{const locale='en';const value=new Intl.NumberFormat(locale,{maximumFractionDigits:28});let checksum=seed;let i=0;while(i<n){checksum+=value.format(input as unknown as number).length;i++;}return checksum;}
