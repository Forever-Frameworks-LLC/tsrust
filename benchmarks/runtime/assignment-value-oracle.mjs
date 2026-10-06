// RHS-only references: ordinary assignment never consults the old target.
import {tokensObservation} from './number-oracle.mjs';
export const intAssignmentRhs=[0,1,2,3,7,31,129,1024];
export const numberAssignmentBits=['c002000000000000','8000000000000000','0000000000000000','3ff8000000000000','4340000000000000','nan','7ff0000000000000','fff0000000000000'];
const booleanTags=[0,1,0,1];
export function validateAssignmentValueCase(id,n,seed){const match=/^(int|number|boolean)_assignment_value$/.exec(id);if(!match)throw Error('Unknown scalar assignment value row');if(!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000)throw Error('Bounded n/seed required');return match[1];}
export function referenceBooleanAssignment(rhs){if(typeof rhs!=='boolean')throw Error('Strict Boolean required');return rhs?[1,1]:[0,0];}
export function expectedAssignmentValueValues(id,n,seed){const family=validateAssignmentValueCase(id,n,seed),pool=family==='number'?numberAssignmentBits:family==='int'?intAssignmentRhs:booleanTags,values=[];let index=seed%pool.length;for(let remaining=n;remaining>0;remaining--){values.push(pool[index],pool[index]);index=(index+13)%pool.length;}return values;}
export function expectedAssignmentValueObservation(id,n,seed){const family=validateAssignmentValueCase(id,n,seed),values=expectedAssignmentValueValues(id,n,seed);if(family==='number')return tokensObservation(values,true);let checksum=0n;for(const code of[256,...values,257])checksum=(checksum*131n+BigInt(code))%1000000007n;return{checksum:Number(checksum),exactBits:'t:['+values.join(',')+']'};}
export const booleanAssignmentHandCells=[false,true].flatMap(old=>[false,true].map(rhs=>({old,rhs,expected:referenceBooleanAssignment(rhs)})));
