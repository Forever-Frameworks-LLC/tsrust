import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../apps/studio/src/', import.meta.url));
const read = (name) => JSON.parse(readFileSync(root + name, 'utf8'));
const english = read('locales/en.json');
const locales = read('locales/names.json');
const keys = Object.keys(english).sort();
const placeholders = (text) => [...text.matchAll(/\{([^{}]+)\}/g)].map((match) => match[1]).sort();
for (const locale of locales) {
  const catalog = read(`locales/${locale}.json`);
  assert.deepEqual(Object.keys(catalog).sort(), keys, `${locale}: catalog keys differ`);
  for (const key of keys) {
    assert.equal(typeof catalog[key], 'string', `${locale}: ${key} is not text`);
    assert.ok(catalog[key].trim(), `${locale}: ${key} is empty`);
    assert.deepEqual(placeholders(catalog[key]), placeholders(english[key]), `${locale}: ${key} placeholders differ`);
  }
}
const known = (key) => Object.hasOwn(english, key) || ['_zero', '_one', '_other'].some((suffix) => Object.hasOwn(english, key + suffix));
for (const file of readdirSync(root).filter((file) => /\.tsrx?$/.test(file))) {
  const source = readFileSync(root + file, 'utf8');
  for (const match of source.matchAll(/\b(?:message|accessibleName|t|tWith|translate|translateWith)\(\s*(['"])([^'"]+)\1/g)) assert.ok(known(match[2]), `${file}: missing message ${match[2]}`);
  for (const match of source.matchAll(/notice\.set\(\s*(['"])([^'"]+)\1\)/g)) assert.ok(known(match[2]), `${file}: missing notice ${match[2]}`);
  for (const match of source.matchAll(/Err\('([^']+)'\)/g)) assert.ok(known(match[1]), `${file}: missing error ${match[1]}`);
  for (const match of source.matchAll(/(?:tooltip|aria-label|placeholder|accessibilityLabel|label)="([^"]+)"/g)) assert.ok(['English', 'Português (Brasil)'].includes(match[1]), `${file}: untranslated prop ${match[1]}`);
  for (const match of source.matchAll(/<(?:Text|Button)\b[^>]*>([^<>{}]+)<\//g)) assert.ok(['Aa', '.*', ''].includes(match[1].trim()), `${file}: untranslated text ${match[1].trim()}`);
}
console.log(`Studio locales: ${locales.length} catalogs, ${keys.length} keys, placeholders and owned UI text verified.`);
