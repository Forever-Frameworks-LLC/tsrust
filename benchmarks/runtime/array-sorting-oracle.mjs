import {observeSliceTree} from './slice-observation.mjs';import {validateArrayRemovalCase} from './array-removal-oracle.mjs';
const defaults=['int','number','boolean','string','null','undefined'],all=['int','number','boolean','string','nested','record','null','undefined'];
export const sortingIds=['sort','to_sorted'].flatMap(m=>[['default',defaults],['compare',all]].flatMap(([mode,fs])=>fs.map(f=>'array_'+m+'_'+mode+'_'+f)));
export function sortingInfo(id){if(!sortingIds.includes(id))throw Error('sorting ID');const [method,mode,family]=id.slice(6).replace('to_sorted','toSorted').split('_');return {method,mode,family};}
export function validateSortingCase(id,n,seed,input){const {family}=sortingInfo(id);input=validateArrayRemovalCase('array_pop_'+family,n,seed,input);if(family==='string')for(const c of input){const p=c.codePointAt(0);if(p>=0xe000&&p<=0xffff)throw Error('UTF8/UTF16 common ordering domain');}return input;}
function digits(v){if(v===0)return'0';let out='';while(v>0){out='0123456789'[v%10]+out;v=Math.floor(v/10);}return out;}
function scalarCompare(a,b){const x=Array.from(a,c=>c.codePointAt(0)),y=Array.from(b,c=>c.codePointAt(0));for(let i=0;i<Math.min(x.length,y.length);i++)if(x[i]!==y[i])return x[i]<y[i]?-1:1;return x.length-y.length;}
// Separate stable insertion reference; never calls tested sort/toSorted methods.
export function expectedSortingValues(id,n,seed,input){input=validateSortingCase(id,n,seed,input);const {method,mode,family:f}=sortingInfo(id),out=[];
 function leaf(j){if(f==='int')return [seed%100+11,2,10,seed%100+1][j%4];if(f==='number')return (mode==='default'?[seed/4,-2.5,0,13.25]:[-0,0,NaN,Infinity,-Infinity,seed/4,-2.5])[(j*3+seed)%(mode==='default'?4:7)];if(f==='boolean')return j%3!==0;if(f==='string')return [input,'é','😀','a'][j%4];if(f==='nested')return [[seed,1],[],[seed+1,2],[seed+2]][j%4];if(f==='record')return {id:(seed+j)%3,label:input+':'+digits(j)};return f==='null'?null:undefined;}
 function cmp(a,b){if(f==='null'||f==='undefined')return 0;if(f==='record')return a.id-b.id;if(f==='nested')return a.length-b.length;if(f==='string')return scalarCompare(a,b);if(f==='number'){if(Number.isNaN(a))return Number.isNaN(b)?0:1;if(Number.isNaN(b))return-1;}return a<b?-1:a>b?1:0;}
 function project(a){if(f==='record')return a.map(v=>digits(v.id)+':'+v.label);if(f==='null')return a.map(()=>'null');if(f==='undefined')return a.map(()=>'undefined');return a;}
 for(let i=0;i<n;i++){const count=[0,1,4,5,17][(i+seed)%5],source=Array.from({length:count},(_,j)=>leaf(j)),sorted=[];for(const item of source){let at=sorted.length;while(at>0&&cmp(sorted[at-1],item)>0)at--;sorted.push(item);for(let j=sorted.length-1;j>at;j--)sorted[j]=sorted[j-1];sorted[at]=item;}if(method==='toSorted')out.push(project(source));out.push(project(sorted));}return out;
}
export function expectedSortingObservation(id,n,seed,input){return observeSliceTree(expectedSortingValues(id,n,seed,input),{numbers:sortingInfo(id).family==='number'});}
