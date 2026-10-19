// Independent application and token arithmetic. No renderer or compiler helpers.
declare const console:{log(value:string):void};
const scopes=[
 {mode:'light',unit:16,base:16,background:'rgb(241, 245, 249)',foreground:'rgb(71, 85, 105)',nested:'rgba(255, 255, 255, 0.2)'},
 {mode:'dark',unit:20,base:20,background:'rgb(30, 41, 59)',foreground:'rgb(203, 213, 225)',nested:'rgba(15, 23, 42, 0.1)'}
].map(value=>({...value,height:5*value.unit/4,minWidth:5*value.unit/4,font:3*value.base/4,lineHeight:value.base,horizontal:1,vertical:12*value.unit/4}));
console.log(JSON.stringify({scopes,live:['1','2','3'],group:'Ctrl+K'}));
