import test from 'node:test';
import assert from 'node:assert/strict';
import {codeTokens, containsCodeSnippet, replaceCodeSnippet, findCodeSnippet} from './source-contract-tokens.mjs';

test('source excerpts accept formatting and retain literals and operators', () => {
    const source = 'function main() {\n  const value = {\n    text: "a b", count: 4\n  };\n  calls += 1;\n}';
    assert.ok(containsCodeSnippet(source, 'const value={text:"a b",count:4};'));
    for (const snippet of ['text:"ab"', 'count:5', 'calls=1;', 'main2()']) {
        assert.equal(containsCodeSnippet(source, snippet), false);
    }
    assert.equal(findCodeSnippet(source, 'calls += 1;').length, 1);
    assert.ok(containsCodeSnippet(replaceCodeSnippet(source, 'text:"a b"', 'text:"changed"'), 'text:"changed"'));
    assert.throws(() => replaceCodeSnippet(source, 'missing()', 'other()'));
});

test('template interpolation ignores code layout while preserving literal chunks', () => {
    const compact='function main(){const message=`A ${ {value:1}.value } B ${2+3}`;const nested=`${`inner ${4}`}`;}';
    const formatted='function main() {\n const message = `A ${{ value: 1 }.value} B ${2 + 3}`;\n const nested = `${`inner ${4}`}`;\n}';
    assert.deepEqual(codeTokens(compact).map(token=>token.text),codeTokens(formatted).map(token=>token.text));
    assert.notDeepEqual(codeTokens(formatted.replace(' B ', ' B  ')).map(token=>token.text),codeTokens(compact).map(token=>token.text));
});


test('snippet token reuse invalidates when source text changes',()=>{
    for(const value of [1,2,1,3,2]) {
        const source=`const value = ${value};`;
        assert.ok(containsCodeSnippet(source,`const value=${value};`));
        assert.equal(containsCodeSnippet(source,`const value=${value+1};`),false);
    }
});
