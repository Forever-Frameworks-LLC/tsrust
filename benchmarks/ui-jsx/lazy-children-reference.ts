// Independent construction order and interactive lifetime model; no renderer helpers.
declare const console:{log(value:string):void};
const trace:string[]=[];
function attribute(){trace.push('attribute');return 'ordered';}
function child(){trace.push('child');return 'Ready';}
function ordered(id:string,children?:()=>string[]){trace.push('parent');return children?.()??[];}
function eagerChild(){trace.push('eager-child');return 'Still eager';}
function eager(children:string[]){trace.push('eager-parent');return children;}
function probe(){trace.push('probe-child');return 'probe';}
function ignore(children?:()=>string[]){trace.push('ignored-parent');return [];}
function repeat(children:()=>string[]){children();children();}
function once(children:()=>string[]){return children();}
function broken():never{trace.push('failed-child');throw new Error('expected');}
ordered(attribute(),()=>[child()]);eager([eagerChild()]);ignore(()=>[probe()]);repeat(()=>[probe()]);once(()=>['Owned child']);try{once(()=>['Owned child',broken()]);}catch{}ordered('empty');ordered('explicit',()=>['Explicit']);
let parent=0,detail:number|null=null,draft='';const steps:{parent:string;child:string|null;draft:string}[]=[];
function snapshot(){steps.push({parent:'Parent count: '+parent,child:detail===null?null:'Detail count: '+detail,draft});}
snapshot();draft='Kept 🦀';parent++;detail=0;snapshot();detail++;detail++;snapshot();detail=null;snapshot();detail=0;snapshot();detail++;parent++;snapshot();detail=null;snapshot();detail=0;snapshot();
console.log(JSON.stringify({nativeTrace:trace,steps}));
