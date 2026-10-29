// Package already-built consumers; never start an implicit release build.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const [compiler, server, target, output, vendor] = process.argv.slice(2);
if (!compiler || !server || !target || !output || process.argv.length > 7)
  throw Error('Usage: node tools/package-toolchain.mjs CLI LSP RUST_TARGET NEW_DIRECTORY [VENDOR_DIRECTORY]');
if (!/^[a-z0-9_]+-[a-z0-9_-]+$/.test(target)) throw Error('Use an explicit Rust target triple.');
const git = (...args) => execFileSync('git', ['-C', root, ...args], {encoding: 'utf8'}).trim();
const version = crate => fs.readFileSync(path.join(root, 'crates', crate, 'Cargo.toml'), 'utf8').match(/^version = "([^"]+)"/m)[1];
const rust = fs.readFileSync(path.join(root, 'rust-toolchain.toml'), 'utf8').match(/^channel = "([^"]+)"/m)[1];
const destination = path.resolve(output);
fs.mkdirSync(destination);
let complete = false;
try {
  const files = {};
  function copy(source, relative) {
    if (!fs.lstatSync(source).isFile()) throw Error(`Expected a regular file: ${source}`);
    const bytes = fs.readFileSync(source);
    const to = path.join(destination, relative);
    fs.mkdirSync(path.dirname(to), {recursive: true});
    fs.writeFileSync(to, bytes);
    files[relative] = {sha256: crypto.createHash('sha256').update(bytes).digest('hex'), bytes: bytes.length};
  }
  const suffix = target.includes('-windows-') ? '.exe' : '';
  for (const [name, input] of [['tsrust', compiler], ['tsrust-lsp', server]]) {
    copy(path.resolve(input), `bin/${name}${suffix}`);
    if (!suffix) fs.chmodSync(path.join(destination, 'bin', name), 0o755);
  }
  copy(path.join(root, 'LICENSE'), 'LICENSE');
  function notices(directory, relative, insideNotice = false) {
    for (const entry of fs.readdirSync(directory, {withFileTypes: true})) {
      if (entry.isSymbolicLink()) throw Error('Notices cannot contain symlinks.');
      const source = path.join(directory, entry.name);
      const to = path.posix.join(relative, entry.name);
      const notice = insideNotice || /^(licen[cs]es?|copying|notices?)(?:[._-]|$)/i.test(entry.name);
      if (entry.isDirectory()) notices(source, to, notice);
      else if (entry.isFile() && (notice || entry.name === 'Cargo.toml')) copy(source, to);
    }
  }
  // Bundled UI component notices belong to the legacy-ui build, not the backend release.
  copy(path.join(root, 'attributions', 'tsrust-MIT.txt'), 'attributions/tsrust-MIT.txt');
  if (vendor) {
    const crates = fs.readdirSync(vendor, {withFileTypes: true});
    if (!crates.length) throw Error('Vendored dependency directory is empty.');
    for (const crate of crates) {
      const directory = path.join(vendor, crate.name);
      if (!crate.isDirectory() || !fs.existsSync(path.join(directory, 'Cargo.toml')) || !fs.existsSync(path.join(directory, '.cargo-checksum.json')))
        throw Error(`Expected a Cargo-vendored crate: ${crate.name}`);
      notices(directory, `attributions/dependencies/${crate.name}`);
    }
  }
  const manifest = {
    schemaVersion: 2, target, version: version('tsrust-compiler'), lspVersion: version('tsrust-lsp'),
    rustToolchain: rust, sourceCommit: git('rev-parse', 'HEAD'),
    sourceTreeDirty: git('status', '--porcelain', '--untracked-files=normal').length > 0,
    distribution: vendor ? 'release' : 'development', files,
  };
  fs.writeFileSync(path.join(destination, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n');
  fs.writeFileSync(path.join(destination, 'rust-toolchain.toml'), `[toolchain]\nchannel = "${rust}"\nprofile = "minimal"\n`);
  fs.writeFileSync(path.join(destination, 'README.md'), `# Tsrust ${manifest.version}\n\nHost: ${target}. Put both executables from bin/ on PATH. Configure the VS Code extension to use this tsrust-lsp. The extension is a separate VSIX release asset.\n\nNative compilation requires Rustup with Rust ${rust} and a system linker; this package does not include Rust itself. Checking, formatting and emission need no external compiler. Prepare backend sources explicitly with tsrust setup --dependencies --yes. Native builds use the shared Cargo cache offline. Portable source exports require explicit tsrust setup --portable --yes preparation; dependency sources are not embedded in these executables.\n\nVerify the archive against the release SHA256SUMS before extraction. manifest.json records consumer and notice hashes, source commit and both consumer versions. Hashes detect corruption, not an untrusted distributor. These executables are not code-signed or notarized.\n\nLicense: MIT; see LICENSE. Third-party notices are in attributions/. Native exported packages retain their own dependency sources and notices. ${vendor ? '' : 'This development package omits dependency notices and is not a redistribution artifact.'}\n\nTsrust is experimental. LSP retention fixes release unrelated editor-buffer snapshots and superseded workspace-scan entries. Bounded Windows and 30-minute macOS probes pass; representative large-workspace and multi-hour memory behavior remains unverified. See the editor troubleshooting guide in the source repository and the release notes for current evidence and limitations.\n`);
  for (const [relative, record] of Object.entries(files)) {
    const digest = crypto.createHash('sha256').update(fs.readFileSync(path.join(destination, relative))).digest('hex');
    if (digest !== record.sha256) throw Error('Copied file digest mismatch.');
  }
  complete = true;
  console.log(destination);
} finally {
  if (!complete) fs.rmSync(destination, {recursive: true, force: true});
}
