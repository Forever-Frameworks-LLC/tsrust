// Tags identify the compiler release; the LSP keeps its own version in the manifest.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
const [tag] = process.argv.slice(2);
assert.equal(process.argv.length, 3, 'Usage: node tools/check-release.mjs vVERSION');
const match = /^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?$/.exec(tag);
assert.ok(match, 'Use a semantic version tag such as v0.1.0 or v0.1.0-rc.1.');
const version = fs.readFileSync('crates/tsrust-compiler/Cargo.toml', 'utf8').match(/^version = "([^"]+)"/m)[1];
assert.equal(tag.slice(1), version, 'Tag must match the compiler Cargo.toml version exactly.');
assert.equal(execFileSync('git', ['status', '--porcelain'], {encoding: 'utf8'}).trim(), '', 'Release source must be clean.');
assert.match(fs.readFileSync('LICENSE', 'utf8'), /^MIT License\n/);
assert.ok(fs.existsSync('Cargo.lock'));
assert.ok(fs.existsSync('crates/tsrust-compiler/native-dependencies/Cargo.lock'));
console.log(`Release source ready: ${tag}`);
