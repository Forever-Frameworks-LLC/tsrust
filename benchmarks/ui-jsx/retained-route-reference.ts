// Independent untimed route lifetime oracle; no compiler/runtime code is imported.
declare const console: {log(value:string):void};
class Route {
 builds=0;value=0;active=false;
 constructor(readonly path:string,readonly exact=true){}
 visit(location:string){const current=location.split('#')[0].replace(/\/$/,'')||'/';this.active=current===this.path||!this.exact&&(this.path==='/'||current.startsWith(this.path+'/'));if(this.active&&this.builds===0)this.builds++;}
}
const route=new Route('/docs');const steps=[];
for(const location of ['/','/docs#start','/','/docs']){route.visit(location);if(location==='/docs#start')route.value++;steps.push({location,builds:route.builds,value:route.value,active:route.active});}
if(route.builds!==1||route.value!==1)throw new Error('Retained route oracle failed');
const prefix=new Route('/docs',false);prefix.visit('/docs-other');if(prefix.active)throw new Error('Prefix crossed a segment');prefix.visit('/docs/api');if(!prefix.active)throw new Error('Missing descendant');
console.log(JSON.stringify({steps,prefix:true}));
