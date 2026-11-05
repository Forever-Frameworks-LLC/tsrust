function workload(n:number,seed:number,input:string):number{let checksum=seed;let i=0;while(i<n){const value=Intl.getCanonicalLocales(input)[0];checksum+=value.length;i++;}return checksum;}
