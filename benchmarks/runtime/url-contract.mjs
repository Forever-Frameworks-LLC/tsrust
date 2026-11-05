import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';

export const urlSpecs = {
 native_url_parse_components: {input: 'https://user:secret@EXAMPLE.com:8443/a/../b?tag=one&tag=two#section', weight: 8558},
 native_url_query_snapshot: {input: 'https://example.com/?tag=one&tag=two&z=last', weight: 59},
 native_url_reused_setters: {input: 'https://example.com/start', weight: 47},
};
for(const multiple of [1,2,4,8])for(const kind of ['long','short']) {
 const size=multiple*8192,authority=kind==='long'?size:1;
 urlSpecs[`native_url_dot_segments_${kind}_${size}`]={input:(kind==='long'?'L':'C').repeat(multiple),weight:2*authority+24};
}

// Closed arithmetic observations of the fixed ASCII output corpus, without URL parsing.
export function urlChecksum(id,n,seed,input) {
 const spec=urlSpecs[id];
 if(!spec||input!==spec.input||!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||!Number.isSafeInteger(seed+n*spec.weight))throw Error('URL oracle inputs');
 return seed+n*spec.weight;
}

export function assertUrlSource(source,row,kind) {
 const spec=urlSpecs[row.id];
 if(!spec||row.family!=='native-url'||!['native','typescript'].includes(kind))throw Error('URL identity');
 if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('URL fingerprint');
 const scaling=row.id.startsWith('native_url_dot_segments_');
 if(row.inputText!==spec.input||row.iterations!==(scaling?1:2000)||row.operationsPerIteration!==1||row.result!=='int')throw Error('URL denominator');
 const required=['let checksum=seed;','while(i<n)','i++;','return checksum;'];
 if(scaling) {
  required.push('const size=input.length*8192;',"const authority=input.startsWith('L')?'a'.repeat(size):'a';","const text='https://'+authority+'@example.com/'+'../'.repeat(size)+'x';","const expected='https://'+authority+'@example.com/x';",kind==='native'?'unwrap(parseUrl(text))':'new URL(text)','href.length!==expected.length','href.startsWith(expected)','value.username.length!==authority.length','value.pathname.length!==2',"value.pathname.startsWith('/x')",'checksum+=href.length+value.username.length+value.pathname.length;');
  if(source.indexOf("const text")>source.indexOf('while')||source.indexOf('const expected')>source.indexOf('while'))throw Error('URL scaling corpus must be prepared outside parse loop');
 } else if(row.id==='native_url_parse_components') {
  required.push(kind==='native'?'unwrap(parseUrl(input))':'new URL(input)', 'href.length!==62', "href.startsWith('https://user:secret@example.com:8443/b?tag=one&tag=two#section')", "host.startsWith('example.com')", 'host.length!==11', "value.pathname.startsWith('/b')", 'value.pathname.length!==2', "value.protocol.startsWith('https:')", 'value.protocol.length!==6', "value.username.startsWith('user')", 'value.username.length!==4', "password.startsWith('secret')", 'password.length!==6', "search.startsWith('?tag=one&tag=two')", 'search.length!==16', "hash.startsWith('#section')", 'hash.length!==8',kind==='native'?'value.port!==8443':"value.port!=='8443'",'checksum+=href.length+host.length+value.pathname.length+value.protocol.length+value.username.length+password.length+search.length+hash.length+8443;');
 } else if(row.id==='native_url_query_snapshot') {
  required.push(kind==='native'?'unwrap(parseUrl(input))':'new URL(input)',kind==='native'?'searchParams(value)':'new URLSearchParams(value.search)',kind==='native'?'params.entries()':'Array.from(params.entries(),([name,value])=>({name,value}))', "params.set('tag','next')", "params.append('q','a b+')", 'params.sort()',kind==='native'?'formatQuery(params)':'params.toString()',"encoded!=='q=a+b%2B&tag=next&z=last'",'rows.length!==3',"rows[0].name!=='tag'","rows[0].value!=='one'","rows[1].name!=='tag'","rows[1].value!=='two'","rows[2].name!=='z'","rows[2].value!=='last'",kind==='native'?'setSearchParams(value,params)':'value.search=params.toString()',"params.delete('tag')",'result.length!==25',"result.startsWith('?q=a+b%2B&tag=next&z=last')",'checksum+=encoded.length+result.length+rows[0].value.length+rows[1].value.length+rows[2].value.length;');
 } else {
  required.push(kind==='native'?'const value=unwrap(parseUrl(input));':'const value=new URL(input);',"value.pathname='/changed/path';","value.search='?q=one+two';",kind==='native'?'value.port=8443;':"value.port='8443';",'href.length!==47',"href.startsWith('https://example.com:8443/changed/path?q=one+two')",'checksum+=href.length;');
 }
 for(const snippet of required)if(!containsCodeSnippet(source,snippet))throw Error('URL observation: '+snippet);
 if(row.id==='native_url_reused_setters'&&source.indexOf(kind==='native'?'parseUrl(input)':'new URL(input)')>source.indexOf('while'))throw Error('URL owner must be reused');
}
