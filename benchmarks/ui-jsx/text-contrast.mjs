// Composite live CSS colors through their actual ancestor backgrounds.
export function textContrasts(evaluate, selector) {
 return evaluate(`(${measure.toString()})(${JSON.stringify(selector)})`);
}
function measure(selector) {
 const canvas=document.createElement('canvas');canvas.width=canvas.height=1;const context=canvas.getContext('2d',{willReadFrequently:true});
 function rgba(color){context.clearRect(0,0,1,1);context.fillStyle=color;context.fillRect(0,0,1,1);return [...context.getImageData(0,0,1,1).data].map((value,index)=>index===3?value/255:value);}
 function over(top,bottom){return top.slice(0,3).map((value,index)=>value*top[3]+bottom[index]*(1-top[3]));}
 function luminance(rgb){return rgb.map(value=>{value/=255;return value<=.04045?value/12.92:((value+.055)/1.055)**2.4;}).reduce((sum,value,index)=>sum+value*[.2126,.7152,.0722][index],0);}
 return Array.from(document.querySelectorAll(selector)).filter(node=>node.getClientRects().length&&!node.closest(':disabled')).map(node=>{
  let ancestors=[];for(let parent=node;parent;parent=parent.parentElement)ancestors.push(parent);let background=[255,255,255];for(const parent of ancestors.reverse())background=over(rgba(getComputedStyle(parent).backgroundColor),background);
  const a=luminance(background),b=luminance(over(rgba(getComputedStyle(node).color),background));return (Math.max(a,b)+.05)/(Math.min(a,b)+.05);
 });
}
