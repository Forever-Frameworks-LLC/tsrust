import fs from 'node:fs';

const identities = JSON.parse(fs.readFileSync(new URL('./format-source-identities.json', import.meta.url), 'utf8'));
const approved = new Set(identities.proofs.map(proof => proof.originalSha256 + ':' + proof.formattedSha256));

// Only registered, token-identical formatting transitions match. Historical
// policies and reference bytes retain their original content identities.
export function sourceIdentityMatches(original, actual) {
    return original === actual || approved.has(original + ':' + actual);
}
