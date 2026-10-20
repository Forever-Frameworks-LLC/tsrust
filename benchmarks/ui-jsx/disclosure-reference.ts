// Independent disclosure state model; no compiler/runtime source is imported.
declare const console:{log(value:string):void};
let first=false,second=false;
const steps:{expanded:boolean[];visible:number}[]=[];
function snapshot(){steps.push({expanded:[first,second],visible:Number(first)+Number(second)});}
snapshot();first=true;snapshot();second=true;snapshot();first=false;snapshot();first=true;snapshot();second=false;snapshot();
console.log(JSON.stringify({steps}));
