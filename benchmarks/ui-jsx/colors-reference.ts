// Independent double-precision oracle, based on CSS Color 4 conversion matrices.
// https://www.w3.org/TR/css-color-4/#color-conversion-code
// Native scene acceptance separately checks its documented premultiplied RGBA8 boundary.
declare const console: {log(value: string): void};
const multiply=(matrix:number[][],vector:number[])=>matrix.map(row=>row.reduce((sum,v,i)=>sum+v*vector[i],0));
function primary() {
 const l=.6,c=.12,h=260*Math.PI/180, lab=[l,c*Math.cos(h),c*Math.sin(h)];
 const roots=multiply([[1,.3963377773761749,.2158037573099136],[1,-.1055613458156586,-.0638541728258133],[1,-.0894841775298119,-1.2914855480194092]],lab);
 const xyz=multiply([[1.2268798758459243,-.5578149944602171,.2813910456659647],[-.0405757452148008,1.112286803280317,-.0717110580655164],[-.0763729366746601,-.4214933324022432,1.5869240198367816]],roots.map(v=>v*v*v));
 const linear=multiply([[12831/3959,-329/214,-1974/3959],[-851781/878810,1648619/878810,36519/878810],[705/12673,-2585/12673,705/667]],xyz);
 return linear.map(v=>Math.abs(v)<=.0031308?v*12.92:Math.sign(v)*(1.055*Math.pow(Math.abs(v),1/2.4)-.055));
}
const rgb=(r:number,g:number,b:number,a=1)=>[r/255,g/255,b/255,a];
console.log(JSON.stringify({
 cells:[
  {slot:'panel',background:rgb(219,234,254,128/255*.5),border:rgb(203,213,225,.5),ring:rgb(23,32,51,.1)},
  {slot:'primary',background:[...primary(),.4]},
  {slot:'transparent',background:[0,0,0,0],ring:rgb(203,213,225)},
  {slot:'alias',background:rgb(23,32,51)},
  {slot:'child-ring',ring:rgb(37,99,235)},
 ], hover:rgb(23,32,51,.9),focus:rgb(37,99,235),
}));
