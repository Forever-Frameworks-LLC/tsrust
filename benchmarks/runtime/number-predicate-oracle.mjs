// Exact rational input specifications; no measured predicate API is used.
const integer=(value)=>({numerator:value,denominator:1n});
const fraction=(numerator,denominator)=>({numerator,denominator});
const huge=(2n**53n-1n)<<971n;
export const predicateInputs=[
 {kind:'nan'},{kind:'infinity',negative:false},{kind:'infinity',negative:true},
 {...integer(0n),negativeZero:true},integer(0n),integer(1n),integer(-1n),fraction(3n,2n),fraction(-3n,2n),
 fraction(1n,2n**1074n),fraction(-1n,2n**1074n),integer(huge),integer(-huge),
 integer(9007199254740991n),integer(-9007199254740991n),integer(9007199254740992n),integer(-9007199254740992n),
 fraction(9007199254740991n,2n),integer(4503599627370496n),integer(4503599627370497n),
 integer(10n),integer(-10n),fraction(1n,4n),fraction(-1n,4n)
];
export function classifyPredicate(method,value) {
 const finite=value.kind===undefined;
 const integral=finite&&value.numerator%value.denominator===0n;
 const magnitude=finite?(value.numerator<0n?-value.numerator:value.numerator):undefined;
 const safe=integral&&magnitude<=9007199254740991n*value.denominator;
 const result={isFinite:finite,isNaN:value.kind==='nan',isInteger:integral,isSafeInteger:safe};
 if(!(method in result))throw new Error('Unknown Number predicate');
 return result[method];
}
const names={number_is_finite:'isFinite',number_is_nan:'isNaN',number_is_integer:'isInteger',number_is_safe_integer:'isSafeInteger'};
export function expectedPredicate(id,n,seed) {
 if(![n,seed].every(value=>Number.isSafeInteger(value)&&value>=0&&value<=1000000))throw new Error('Bounded predicate inputs required');
 const method=names[id];if(!method)throw new Error('Unknown Number predicate');
 let digest=BigInt(seed);
 for(let i=0;i<n;i++)digest=(digest*131n+(classifyPredicate(method,predicateInputs[(i*13+seed)%24])?1n:0n))%1000000007n;
 return Number(digest);
}
