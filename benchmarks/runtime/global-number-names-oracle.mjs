import {observeSliceTree} from './slice-observation.mjs';
export const globalNumberNamesIds=['global_number_names'];
// Independent of the platform's number readers and tests. Text is read digit
// by digit, and not-a-number is detected by self-inequality.
function leadingWhole(text){
 let at=0;while(at<text.length&&text[at]===' ')at++;
 let sign=1;if(text[at]==='+'||text[at]==='-'){if(text[at]==='-')sign=-1;at++;}
 let value=0,digits=0;
 for(;at<text.length;at++){const digit=text.charCodeAt(at)-48;if(digit<0||digit>9)break;value=value*10+digit;digits++;}
 return digits?sign*value:0/0;
}
export function expectedGlobalNumberNamesValues(id,n,seed,input){
 if(!globalNumberNamesIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Global number name case');
 const missing=leadingWhole(input),absent=missing!==missing;
 const result=[];for(let i=0;i<n;i++){const base=(i*13+seed)%97;
  // The decimal digits of base, read again in base sixteen.
  const hex=base<10?base:((base-base%10)/10)*16+base%10;
  result.push([''+(base+hex),''+(base+0.25),absent?'true':'false','true',absent?'NaN':''+(missing+1)]);}
 return result;
}
export function expectedGlobalNumberNamesObservation(id,n,seed,input){return observeSliceTree(expectedGlobalNumberNamesValues(id,n,seed,input));}
