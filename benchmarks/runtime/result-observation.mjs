import {isCopying,assertCopyingPolicy,validateCopyingObservation} from './copying-array-observation.mjs';
import {isRange,assertRangePolicy,validateRangeObservation} from './range-mutation-observation.mjs';
import {isHigher} from './higher-order-observation.mjs';
import {isDeclared,assertDeclaredPolicy,validateDeclaredObservation} from './declared-value-observation.mjs';
import {isReductionSchema,validateReductionObservation} from './reduction-observation.mjs';
import {isNumberResult,validateNumberObservation} from './number-observation.mjs';
export function isSliceResult(workload){return workload.resultEncoding==='array-shape-scalar-v1'||workload.resultEncoding==='ieee-f64-tree-v1';}
export function requiresExactResult(workload){return isCopying(workload)||isRange(workload)||isHigher(workload)&&workload.result!=='int'||isDeclared(workload)&&workload.result!=='int'||isNumberResult(workload)||isSliceResult(workload)||isReductionSchema(workload);}
function validateTreeToken(text){
 const floating=text.startsWith('m:');if(!floating&&!text.startsWith('t:'))throw new Error('Invalid exact array result');
 let index=2;
 const fail=()=>{throw new Error('Invalid exact array result');};
 function integer(){const match=/^(0|[1-9][0-9]*)/.exec(text.slice(index));if(!match)fail();index+=match[0].length;const value=Number(match[0]);if(!Number.isSafeInteger(value))fail();return value;}
 function value(depth){
  if(depth>4)fail();
  if(text[index]==='['){index++;if(text[index]!==']'){value(depth+1);while(text[index]===','){index++;value(depth+1);}}if(text[index++]!==']')fail();return;}
  if(!floating&&text[index]==='b'){if(!/^b[01](?=[,\]])/.test(text.slice(index)))fail();index+=2;return;}
  if(floating){const match=/^(nan|[0-9a-f]{16})/.exec(text.slice(index));if(!match)fail();index+=match[0].length;if(match[0]!=='nan'){const bits=BigInt('0x'+match[0]);if((bits&0x7ff0000000000000n)===0x7ff0000000000000n&&(bits&0x000fffffffffffffn)!==0n)fail();}return;}
  if(text.slice(index,index+2)==='s('){index+=2;if(text[index]!==')'){const scalar=()=>{const code=integer();if(code>0x10ffff||code>=0xd800&&code<=0xdfff)fail();};scalar();while(text[index]===','){index++;scalar();}}if(text[index++]!==')')fail();return;}
  integer();
 }
 if(text[index]!=='[')fail();value(0);if(index!==text.length)fail();
}
export function validateExactObservation(actual,expected){
 if(expected?.exactBits?.startsWith('r:'))return validateReductionObservation(actual,expected);
 if(typeof expected?.exactBits==='string'&&(expected.exactBits.startsWith('m:')||expected.exactBits.startsWith('t:'))){
  if(!Number.isSafeInteger(actual?.checksum)||typeof actual?.exactBits!=='string')throw new Error('Invalid exact array result');validateTreeToken(actual.exactBits);validateTreeToken(expected.exactBits);
  if(actual.checksum!==expected.checksum||actual.exactBits!==expected.exactBits)throw new Error('Exact array result disagrees with independent oracle');
 }else validateNumberObservation(actual,expected);
}
