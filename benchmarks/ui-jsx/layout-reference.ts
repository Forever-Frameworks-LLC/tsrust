// Independent geometry oracle for the authored example; no renderer implementation imports.
declare const console: {log(value: string): void};
const padding=20, gap=20, height=40;
const observed=[360,640,320].map(width=>{
 const track=(width-padding*2-gap)/2;
 return {width,cells:[
  {slot:'a',x:padding,y:padding,width:track,height},
  {slot:'b',x:padding+track+gap,y:padding,width:track,height:height*2+gap},
  {slot:'c',x:padding,y:padding+height+gap,width:track,height},
 ]};
});
console.log(JSON.stringify(observed));
