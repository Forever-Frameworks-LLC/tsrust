// Literal decoded scalar expectations are declared independently of the source lexemes.
import fs from 'node:fs';
import {observeSliceTree} from './slice-observation.mjs';
const domains=JSON.parse(fs.readFileSync(new URL('quoted-string-literals-domains.json',import.meta.url)));
export const quotedStringLiteralIds=['quoted_string_literals_ascii','quoted_string_literals_unicode'];
export function validateQuotedStringLiteralCase(id,n,seed,input){if(!quotedStringLiteralIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000)throw Error('Known bounded quoted-literal case');const family=id.endsWith('_ascii')?'ascii':'unicode';input??=domains[family].inputText;if(typeof input!=='string')throw Error('String input');for(const scalar of input){const cp=scalar.codePointAt(0);if(cp>=0xd800&&cp<=0xdfff)throw Error('Lone surrogate excluded');if(family==='ascii'&&cp>127)throw Error('ASCII input required');}if(new TextEncoder().encode(input).length>256)throw Error('Input at most256 UTF8 bytes');return {family,input};}
export function expectedQuotedStringLiteralValues(id,n,seed,input){const c=validateQuotedStringLiteralCase(id,n,seed,input),pool=[c.input,...domains[c.family].decoded],out=[];let at=seed%16;for(let remaining=n;remaining>0;remaining--){out.push(pool[at]);at=(at+13)%16;}return out;}
export function expectedQuotedStringLiteralObservation(id,n,seed,input){return observeSliceTree(expectedQuotedStringLiteralValues(id,n,seed,input));}
