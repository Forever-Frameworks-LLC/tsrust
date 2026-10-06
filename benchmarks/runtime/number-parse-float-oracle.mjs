// Expected IEEE values are literal facts; this module does not parse numeric text.
import fs from 'node:fs';import {observeNumber} from './number-observation.mjs';
export const numberParseFloatDomains=JSON.parse(fs.readFileSync(new URL('number-parse-float-domains.json',import.meta.url)));
export const numberParseFloatIds=Object.keys(numberParseFloatDomains.workloadPools);
const cells=new Map(numberParseFloatDomains.cells.map(cell=>[cell.id,cell])),inputs=new Map(numberParseFloatDomains.cells.map(cell=>[cell.input,cell]));
export function numberParseFloatLiteralValue(cell){if(cell.bits==='nan')return NaN;const view=new DataView(new ArrayBuffer(8));view.setBigUint64(0,BigInt('0x'+cell.bits),false);return view.getFloat64(0,false);}
export function validateNumberParseFloatCase(id,n,seed,input){if(!numberParseFloatIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string'||!numberParseFloatDomains.externalInputs[id].includes(input)||input.includes('\u0000'))throw Error('Bounded known parsing row and declared NUL-free input required');}
export function expectedNumberParseFloatValues(id,n,seed,input){validateNumberParseFloatCase(id,n,seed,input);const pool=numberParseFloatDomains.workloadPools[id].map(key=>cells.get(key));pool[0]=inputs.get(input);const out=[];for(let i=0;i<n;i++)out.push(numberParseFloatLiteralValue(pool[(i*13+seed)%pool.length]));return out;}
export function expectedNumberParseFloatObservation(id,n,seed,input){return observeNumber(expectedNumberParseFloatValues(id,n,seed,input));}
