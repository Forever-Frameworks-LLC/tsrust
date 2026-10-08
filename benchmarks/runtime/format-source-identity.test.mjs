import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {sourceIdentityMatches} from './format-source-identity.mjs';
import {codeTokens} from './source-contract-tokens.mjs';

const hash = value => createHash('sha256').update(value).digest('hex');
const proofs = JSON.parse(fs.readFileSync(new URL('./format-source-identities.json', import.meta.url))).proofs;
test('formatting transitions bind current bytes, exact tokens and immutable original identities', () => {
    assert.ok(proofs.length > 0);
    for (const proof of proofs) {
        const source = fs.readFileSync(new URL('../../' + proof.path, import.meta.url), 'utf8');
        assert.equal(hash(source), proof.formattedSha256);
        assert.equal(hash(JSON.stringify(codeTokens(source, true).map(token => token.text))), proof.tokenSha256);
        assert.equal(sourceIdentityMatches(proof.originalSha256, hash(source)), true);
        assert.equal(sourceIdentityMatches(proof.originalSha256, hash(source + '\n')), false);
        assert.equal(sourceIdentityMatches(proof.originalSha256, hash(source + '\nconst changed = 1;')), false);
        assert.equal(sourceIdentityMatches('0'.repeat(64), hash(source)), false);
    }
});
