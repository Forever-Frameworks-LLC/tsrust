import {sourceIdentityMatches} from './format-source-identity.mjs';
import {isCopying,assertCopyingPolicy,validateCopyingObservation} from './copying-array-observation.mjs';
import {isRange,assertRangePolicy,validateRangeObservation} from './range-mutation-observation.mjs';
import {isHigher,assertHigherPolicy} from './higher-order-observation.mjs';
// Proposed benchmark-only validator. No Math operation is used as its own oracle.
import {assertDeclaredPolicy} from './declared-value-observation.mjs';
import {createHash} from 'node:crypto';
import {rationalBits} from './number-oracle.mjs';
export const APPROXIMATE_SCALAR_PROTOCOL='mpfr-directed-f64-scalar-v1';
const encodedResult='approximate-f64-scalar-v1';
const methods=new Set(['acos','acosh','asin','asinh','atan','atanh','cbrt','cos','cosh','exp','expm1','log','log10','log1p','log2','sin','sinh','tan','tanh','atan2','pow','hypot']);
const hex=/^[0-9a-f]{16}$/,sha=/^[0-9a-f]{64}$/;
const mask=(1n<<64n)-1n,sign=1n<<63n,denominator=1n<<1074n;
function fail(message){throw new Error('Approximate scalar protocol: '+message);}
function bits(value){if(typeof value!=='string'||!hex.test(value))fail('raw lowercase binary64 hex required');return BigInt('0x'+value);}
function finite(value){const raw=bits(value);if((raw&0x7ff0000000000000n)===0x7ff0000000000000n)fail('finite bounded-domain result required');return raw;}
function rank(value){const raw=finite(value);return raw&sign?(~raw)&mask:raw|sign;}
function fromRank(value){const raw=value&sign?value&(~sign):(~value)&mask;const result=raw.toString(16).padStart(16,'0');finite(result);return result;}
function units(value){const raw=finite(value),exponent=(raw>>52n)&2047n,fraction=raw&0xfffffffffffffn;const magnitude=exponent===0n?fraction:((1n<<52n)|fraction)<<(exponent-1n);return raw&sign?-magnitude:magnitude;}
function roundedAdd(left,right){const negativeZero=left==='8000000000000000'&&right==='8000000000000000';const result=rationalBits(units(left)+units(right),denominator,negativeZero);finite(result);return result;}
function dyadic(endpoint){
 if(!endpoint||typeof endpoint.numerator!=='string'||endpoint.numerator.length>10000||!/^-(?:[1-9][0-9]*)$|^(?:0|[1-9][0-9]*)$/.test(endpoint.numerator))fail('directed dyadic numerator');
 const power=checkedInteger(endpoint.denominatorPowerOfTwo,16384,'directed dyadic denominator');return {n:BigInt(endpoint.numerator),d:1n<<BigInt(power)};
}
function directedNearest(point){
 const lower=dyadic(point.directedLower),upper=dyadic(point.directedUpper);if(lower.n*upper.d>upper.n*lower.d)fail('reversed MPFR interval');
 const negativeZero=point.zeroSign==='negative';
 if(rationalBits(lower.n,lower.d,negativeZero)!==point.nearestBits||rationalBits(upper.n,upper.d,negativeZero)!==point.nearestBits)fail('MPFR endpoints do not establish unique nearest result');
}
function interval(lower,upper){finite(lower);finite(upper);if(rank(lower)>rank(upper))fail('reversed interval');return {lower,upper};}
function checkedInteger(value,max,label){if(!Number.isSafeInteger(value)||value<0||value>max)fail(label);return value;}
function zeroSign(value,required){if(required!=='either'&&required!=='positive'&&required!=='negative')fail('zero-sign policy required');if((finite(value)&(~sign))===0n&&required!=='either'&&(value[0]==='8')!==(required==='negative'))fail('signed zero disagrees with contract');}
export function validationProtocol(workload){
 if(isCopying(workload)){assertCopyingPolicy(workload);return 'legacy-exact';}
 if(isRange(workload)){assertRangePolicy(workload);return 'legacy-exact';}
 if(isHigher(workload)){assertHigherPolicy(workload);return 'legacy-exact';}
 if(workload.resultEncoding===encodedResult){if(workload.result!=='number'||workload.inputText!==undefined||workload.scalarCoordinates||workload.approximateValidation?.protocol!==APPROXIMATE_SCALAR_PROTOCOL)fail('explicit scalar protocol selection required');return APPROXIMATE_SCALAR_PROTOCOL;}
 if(workload.resultEncoding==='declared-value-result-v1'){assertDeclaredPolicy(workload);if(!['boolean','string[]','string[][]','Row[][]','Node[]'].includes(workload.result))fail('unknown declared exact result');return 'legacy-exact';}
 if(workload.resultEncoding==='reduction-schema-v1'&&workload.reductionObserver==='declared-result-v1'&&workload.approximateValidation===undefined&&['boolean','string','Row','boolean[][]','Row[][]'].includes(workload.result))return 'legacy-exact';
 if(workload.approximateValidation!==undefined||![undefined,'ieee-f64-bits-v1','array-shape-scalar-v1','ieee-f64-tree-v1'].includes(workload.resultEncoding))fail('unknown or inconsistent result protocol');
 return 'legacy-exact';
}
function sameJSON(left,right){
 const sorted=value=>Array.isArray(value)?value.map(sorted):value&&typeof value==='object'?Object.fromEntries(Object.keys(value).sort().map(key=>[key,sorted(value[key])])):value;
 return JSON.stringify(sorted(left))===JSON.stringify(sorted(right));
}
function hashedJSON(bytes,expectedHash,label){
 if(!(bytes instanceof Uint8Array)||bytes.byteLength===0||bytes.byteLength>16*1024*1024||!sha.test(expectedHash))fail(label+' actual bytes/hash required');
 const actualHash=createHash('sha256').update(bytes).digest('hex');if(actualHash!==expectedHash)fail(label+' artifact identity mismatch');
 try{return JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(bytes));}catch{fail(label+' valid UTF8 JSON required');}
}
export function approvedPolicyForWorkload(workload,{approvedPolicyBytes,inputRecipeBytes,tsSha256,tsrSha256}){
 if(validationProtocol(workload)!==APPROXIMATE_SCALAR_PROTOCOL)fail('policy attached to exact workload');
 const spec=workload.approximateValidation,document=hashedJSON(approvedPolicyBytes,spec.policySha256,'approved policy');
 if(document.schemaVersion!==1||document.protocol!==APPROXIMATE_SCALAR_PROTOCOL||document.predeclared!==true||!document.policyByWorkload||!Object.hasOwn(document.policyByWorkload,workload.id))fail('independently approved workload policy required');
 const approved=document.policyByWorkload[workload.id];
 if(!methods.has(approved.method)||approved.method!==spec.method)fail('approved method mismatch');
 const arity=approved.method==='hypot'?4:['atan2','pow'].includes(approved.method)?2:1;
 if(approved.operationArity!==arity||spec.operationArity!==arity)fail('approved method arity mismatch');
 const maximum=approved.method==='hypot'?32:8;
 if(!approved.policy||approved.policy.predeclared!==true||approved.policy.maxOperationRankSteps!==maximum||approved.policy.exactSupportingAddition!=='binary64-nearest-even-in-source-order'||approved.policy.quantization!==false)fail('root-approved 8/32 numerical policy required');
 if(!approved.domain)fail('approved domain required');checkedInteger(approved.domain.maxIterations,1000000,'approved iteration domain');checkedInteger(approved.domain.maxSeed,1000000,'approved seed domain');
 if(!sourceIdentityMatches(approved.tsSha256,tsSha256)||!sourceIdentityMatches(approved.tsrSha256,tsrSha256)||!sha.test(tsSha256)||!sha.test(tsrSha256))fail('approved source identity mismatch');
 hashedJSON(inputRecipeBytes,approved.inputRecipeSha256,'approved input recipe');
 if(!Array.isArray(approved.argumentBounds)||approved.argumentBounds.length!==arity)fail('approved argument bounds required');
 for(const bound of approved.argumentBounds){const lower=dyadic(bound.lower),upper=dyadic(bound.upper);if(lower.n*upper.d>upper.n*lower.d)fail('reversed approved argument bounds');}
 return approved;
}
function approvedRecipePoints(workload,approved,inputRecipeBytes){
 const document=hashedJSON(inputRecipeBytes,approved.inputRecipeSha256,'approved input recipe'),recipe=document[approved.method];
 if(!recipe||recipe.method!==approved.method||recipe.workloadId!==workload.id||recipe.operationArity!==approved.operationArity||recipe.maxIterations!==approved.domain.maxIterations||recipe.maxSeed!==approved.domain.maxSeed||recipe.tableSelector!=='(i*13+seed)%8'||recipe.jitterSelector!=='(i*17+seed)%5'||recipe.tuplesPerMethod!==40||recipe.iterationPeriod!==40||!Array.isArray(recipe.baseRationals)||recipe.baseRationals.length!==approved.operationArity||!Array.isArray(recipe.jitterDenominators)||recipe.jitterDenominators.length!==approved.operationArity)fail('unsupported or mismatched approved input recipe');
 const points=new Map();
 for(let position=0;position<8;position++)for(let jitter=0;jitter<5;jitter++){
  const inputBits=recipe.baseRationals.map((table,index)=>{
   if(!Array.isArray(table)||table.length!==8)fail('approved eight-position dyadic table required');
   const base=dyadic(table[position]),divisor=recipe.jitterDenominators[index];
   if(!Number.isSafeInteger(divisor)||divisor<1||divisor>65536||(divisor&(divisor-1))!==0)fail('approved dyadic jitter divisor required');
   return rationalBits(base.n*BigInt(divisor)+BigInt(jitter)*base.d,base.d*BigInt(divisor));
  });points.set(position+':'+jitter,inputBits);
 }return points;
}
export function validateReferenceBundle(referenceBytes,workload,context){
 const approved=approvedPolicyForWorkload(workload,context),spec=workload.approximateValidation;
 if(spec.referencePending===true)fail('reference generation pending');
 const bundle=hashedJSON(referenceBytes,spec.referenceSha256,'reference');
 if(bundle.protocol!==APPROXIMATE_SCALAR_PROTOCOL||bundle.schemaVersion!==1||bundle.workloadId!==workload.id||!sourceIdentityMatches(bundle.tsSha256,context.tsSha256)||!sourceIdentityMatches(bundle.tsrSha256,context.tsrSha256)||bundle.policySha256!==spec.policySha256)fail('reference source identity mismatch');
 if(bundle.method!==approved.method||bundle.operationArity!==approved.operationArity||bundle.inputRecipeSha256!==approved.inputRecipeSha256||!sameJSON(bundle.policy,approved.policy)||!sameJSON(bundle.domain,approved.domain)||!sameJSON(bundle.argumentBounds,approved.argumentBounds))fail('reference does not match actual approved policy/domain/method/arity/bounds');
 const recipePoints=approvedRecipePoints(workload,approved,context.inputRecipeBytes);
 const provenance=bundle.provenance;
 if(!provenance||provenance.gmpy2!=='2.3.1'||provenance.mpfr!=='4.2.2'||provenance.gmp!=='GMP 6.3.0'||provenance.precisionBits<256||!Number.isSafeInteger(provenance.precisionBits)||provenance.lowerRounding!=='RoundDown'||provenance.upperRounding!=='RoundUp'||!sha.test(provenance.generatorSha256)||provenance.dependencySha256!=='63c867ad5bf43486dc4d7e4b6bafb04bd2b50bce400130f4c49fa4bf4325076e'||!Array.isArray(provenance.command)||!provenance.command.length)fail('directed reference provenance missing');
 if(!Array.isArray(bundle.points)||bundle.points.length!==recipePoints.size||!Array.isArray(bundle.cases)||!bundle.cases.length)fail('ordinary finite point/case corpus missing');
 const points=new Map(),tuples=new Set();
 for(const point of bundle.points){
  if(typeof point.id!=='string'||points.has(point.id)||!Array.isArray(point.inputBits)||point.inputBits.length!==spec.operationArity)fail('invalid unique point');
  if(!sameJSON(point.inputBits,recipePoints.get(point.id)))fail('point ID/input bits disagree with approved recipe');
  for(let index=0;index<point.inputBits.length;index++){const value=units(point.inputBits[index]),bound=approved.argumentBounds[index],lower=dyadic(bound.lower),upper=dyadic(bound.upper);if(value*lower.d<lower.n*denominator||value*upper.d>upper.n*denominator)fail('point outside approved argument bounds');}tuples.add(point.inputBits.join(','));
  const width=checkedInteger(point.allowedRankSteps,bundle.policy.maxOperationRankSteps,'point bound');const center=rank(point.nearestBits);
  const expectedWidth=(finite(point.nearestBits)&(~sign))===0n?0:approved.policy.maxOperationRankSteps;if(width!==expectedWidth)fail('point does not use approved uniform allowance');
  if(point.uniqueNearestVerified!==true||!Number.isSafeInteger(point.refinementPrecisionBits)||point.refinementPrecisionBits<provenance.precisionBits)fail('unresolved nearest reference');
  directedNearest(point);
  if((finite(point.nearestBits)&(~sign))===0n&&width!==0)fail('exact zero point must retain its contract');
  // These closed endpoints are predeclared, never fitted to an engine output.
  const lower=fromRank(center-BigInt(width)),upper=fromRank(center+BigInt(width));
  if(point.acceptedLowerBits!==lower||point.acceptedUpperBits!==upper)fail('point allowance certificate mismatch');
  zeroSign(point.nearestBits,point.zeroSign);points.set(point.id,point);
 }
 if(tuples.size<12)fail('distinct runtime-derived input corpus missing');
 const cases=new Map();
 for(const reference of bundle.cases){
  checkedInteger(reference.iterations,bundle.domain.maxIterations,'reference iterations');checkedInteger(reference.seed,bundle.domain.maxSeed,'reference seed');
  const key=reference.iterations+':'+reference.seed;if(cases.has(key)||!Array.isArray(reference.steps)||reference.steps.length!==reference.iterations)fail('invalid case/ordered operation count');
  const initial=rationalBits(BigInt(reference.seed),1n);if(reference.initialBits!==initial)fail('case initial value disagrees with approved recipe');
  for(let index=0;index<reference.steps.length;index++){const i=BigInt(index),seed=BigInt(reference.seed),expected=((i*13n+seed)%8n)+':'+((i*17n+seed)%5n);if(reference.steps[index]!==expected)fail('case operation order disagrees with approved recipe');}
  let lower=reference.initialBits,upper=reference.initialBits,ideal=reference.initialBits;finite(ideal);
  for(const id of reference.steps){const point=points.get(id);if(!point)fail('case uses unknown point');lower=roundedAdd(lower,point.acceptedLowerBits);upper=roundedAdd(upper,point.acceptedUpperBits);ideal=roundedAdd(ideal,point.nearestBits);interval(lower,upper);}
  if(reference.acceptedLowerBits!==lower||reference.acceptedUpperBits!==upper||reference.idealAccumulatorBits!==ideal)fail('ordered supporting-addition certificate mismatch');
  zeroSign(ideal,reference.zeroSign);cases.set(key,reference);
 }
 return {bundle,approvedPolicy:approved,points,cases};
}
function validateValue(actual,reference){
 finite(actual);const actualRank=rank(actual),low=rank(reference.acceptedLowerBits),high=rank(reference.acceptedUpperBits);
 if(actualRank<low||actualRank>high)fail('actual output outside independent accepted interval');
 zeroSign(actual,reference.zeroSign);
 const ideal=reference.idealAccumulatorBits??reference.nearestBits,idealRank=rank(ideal);const distance=actualRank>=idealRank?actualRank-idealRank:idealRank-actualRank;
 return {actualBits:actual,referenceBits:ideal,acceptedLowerBits:reference.acceptedLowerBits,acceptedUpperBits:reference.acceptedUpperBits,binary64RankDistance:distance.toString(),exactDifferenceFromRoundedReference:{numerator:(units(actual)-units(ideal)).toString(),denominatorPowerOfTwo:1074}};
}
export function validateApproximatePointProbe(actual,reference){
 if(actual?.protocol!==APPROXIMATE_SCALAR_PROTOCOL||!Array.isArray(actual.pointBits)||actual.pointBits.length!==reference.bundle.points.length)fail('all-point probe required');
 return actual.pointBits.map((value,index)=>({pointId:reference.bundle.points[index].id,...validateValue(value,reference.bundle.points[index])}));
}
export function validateApproximateMeasurement(actual,reference,{iterations,seed,warmups,samples,pointProbeAccepted}){
 if(pointProbeAccepted!==true)fail('engine point probe must pass before scalar timing');
 const ref=reference.cases.get(iterations+':'+seed);if(!ref)fail('case has no immutable reference');
 if(actual?.protocol!==APPROXIMATE_SCALAR_PROTOCOL||!Array.isArray(actual.actualBits)||!Array.isArray(actual.warmupBits)||!Array.isArray(actual.durations)||actual.actualBits.length!==samples||actual.warmupBits.length!==warmups||actual.durations.length!==samples||actual.durations.some(t=>!Number.isFinite(t)||t<0))fail('invalid measurement shape/count/durations');
 return {protocol:APPROXIMATE_SCALAR_PROTOCOL,referenceCase:iterations+':'+seed,warmups:actual.warmupBits.map(value=>validateValue(value,ref)),samples:actual.actualBits.map(value=>validateValue(value,ref))};
}

// Untimed allocation evidence has its own contract; it is never a timing sample.
export function validateApproximateAllocation(actual,reference,{iterations,seed,acceptedNativePointProbe}){
 if(!Array.isArray(acceptedNativePointProbe)||acceptedNativePointProbe.length!==reference.bundle.points.length)fail('accepted native all-point probe required for allocation evidence');
 // Revalidate every saved native point result, rather than trusting a boolean flag.
 validateApproximatePointProbe({protocol:APPROXIMATE_SCALAR_PROTOCOL,pointBits:acceptedNativePointProbe.map(point=>point.actualBits)},reference);
 const ref=reference.cases.get(iterations+':'+seed);if(!ref)fail('allocation case has no immutable reference');
 if(actual?.protocol!==APPROXIMATE_SCALAR_PROTOCOL||actual.kind!=='native-approximate-allocation-result-v1'||actual.iterations!==iterations||actual.seed!==seed||Object.hasOwn(actual,'durations')||Object.hasOwn(actual,'checksum'))fail('distinct untimed allocation payload required');
 for(const name of ['allocations','reallocations','deallocationsDuringWork','requestedBytes','retainedRequestedBytes','peakRequestedBytes','inputRequestedBytes'])checkedInteger(actual[name],Number.MAX_SAFE_INTEGER,'allocation counter '+name);
 if(actual.liveBytesAfterOutputDrop!==0||actual.liveBytesAfterDrop!==0)fail('allocation disposal must be zero');
 return {protocol:APPROXIMATE_SCALAR_PROTOCOL,kind:actual.kind,referenceCase:iterations+':'+seed,result:validateValue(actual.actualBits,ref),counters:Object.fromEntries(['allocations','reallocations','deallocationsDuringWork','requestedBytes','retainedRequestedBytes','peakRequestedBytes','inputRequestedBytes','liveBytesAfterOutputDrop','liveBytesAfterDrop'].map(name=>[name,actual[name]]))};
}
