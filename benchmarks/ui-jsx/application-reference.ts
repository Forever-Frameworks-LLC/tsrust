// Independent state-transition oracle; no renderer/compiler implementation imports.
declare const console:{log(value:string):void};
let count=0;let draft='Original';let mode='light';let source=false;
const steps:unknown[]=[];
function snapshot(){steps.push({count:`Count: ${count}`,draft,mode:`Mode: ${mode}`,source,background:mode==='light'?'rgb(255, 255, 255)':'rgb(15, 23, 42)'});}
snapshot();count++;draft='Edited 🦀';mode='dark';snapshot();source=true;snapshot();mode='light';snapshot();source=false;snapshot();
console.log(JSON.stringify({steps,nested:'rgb(255, 255, 255)'}));
