// Independent empty-state variant/size model; no compiler or UI runtime imports.
declare const console:{log(value:string):void};
function state(variant:string|null,large:boolean){return {variant,defaultSize:variant==='icon'?16:24,explicitSize:large?24:16};}
console.log(JSON.stringify({steps:[state('icon',false),state(null,false),state(null,true),state('default',true),state('icon',true)]}));
