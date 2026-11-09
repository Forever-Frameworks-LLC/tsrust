// Package already-built consumers; never start an implicit release build.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const SUPPLEMENTS = [
  ['valuable', '0.1.1', 'ba73ea9cf16a25df0c8caa16c51acb937d5712a8429db78a3ee29d5dcacd3a65', 'valuable-0.1.1-MIT.txt', 'ed60d479b8fd1f64e9cbc3de449a16a53ac1b3d1b6aeb9bf9d190a8e93061b44'],
  ['jni', '0.22.4', '5efd9a482cf3a427f00d6b35f14332adc7902ce91efb778580e180ff90fa3498', 'jni-0.22.4-MIT.txt', 'fea1d5bf3dd71605ce5d7d2ff695c1837e914c77195e523a86b5391716477960'],
  ['jni-macros', '0.22.4', 'a00109accc170f0bdb141fed3e393c565b6f5e072365c3bd58f5b062591560a3', 'jni-0.22.4-MIT.txt', 'fea1d5bf3dd71605ce5d7d2ff695c1837e914c77195e523a86b5391716477960'],
  ['jni-sys-macros', '0.4.1', '38c0b942f458fe50cdac086d2f946512305e5631e720728f2a61aabcd47a6264', 'jni-sys-macros-0.4.1-MIT.txt', '1d85bd754b04ceec93e98e890edd1fa3c6a22e81bcb32135806beeccefa51cd1'],
  ['rustls-platform-verifier-android', '0.2.0', 'eec689c0bc40ff2458a5977b6619cb718087084a18e02a131c599b62d05e1a5f', 'rustls-platform-verifier-android-0.2.0-MIT.txt', '1c7cf76689c837a68ed8d704994e52a0f2940c087958f860d17f3186afbdcc0c'],
  ['rsqlite-vfs', '0.1.1', 'c51c9ae4df8a7fba42103df5c621fa3c37eccf3a3c650879e90fc48b11cc192c', 'rsqlite-vfs-0.1.1-MIT.txt', 'e1c1975c3474cce31722836f101a40ffaf1cc784e4d2f38ce3e77b9712934e78'],
];

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
  function notices(directory, relative, insideNotice = false, authorsNotice = null) {
    for (const entry of fs.readdirSync(directory, {withFileTypes: true})) {
      if (entry.isSymbolicLink()) throw Error('Notices cannot contain symlinks.');
      const source = path.join(directory, entry.name);
      const to = path.posix.join(relative, entry.name);
      const notice = insideNotice || /^(licen[cs]es?|copying|notices?)(?:[._-]|$)/i.test(entry.name);
      if (entry.isDirectory()) notices(source, to, notice, authorsNotice);
      else if (entry.isFile() && (notice || entry.name === 'Cargo.toml' || to === authorsNotice)) copy(source, to);
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
      // Cargo vendor normally emits a normalized literal [package] table. Scope
      // identity to that table; a [lib] or dependency name is not the package.
      const packageTable = fs.readFileSync(path.join(directory, 'Cargo.toml'), 'utf8').split(/^\[package\][ \t]*\r?$/m)[1]?.split(/^\[/m)[0] ?? '';
      const packageName = /^name[ \t]*=[ \t]*"([^"]+)"[ \t]*(?:#.*)?\r?$/m.exec(packageTable)?.[1];
      const packageVersion = /^version[ \t]*=[ \t]*"([^"]+)"[ \t]*(?:#.*)?\r?$/m.exec(packageTable)?.[1];
      let authorsNotice = null;
      if (packageName === 'r-efi' || ['r-efi','r-efi-5.3.0','r-efi-6.0.0'].includes(crate.name)) {
        if (packageName !== 'r-efi') throw Error('Expected Cargo-normalized r-efi package identity.');
        const expected = packageVersion === '5.3.0' ? 'ff92bed461f50338dd703a9ba9aee496a425957873df1d91192776e4bdf5dda7' :
          packageVersion === '6.0.0' ? 'd027e91dbc9cdbb2f1190068e498bd6b61cff022b6a032b191021ba658d96111' : null;
        if (!expected) throw Error('Unreviewed r-efi package version for AUTHORS MIT notice.');
        if (crate.name !== 'r-efi' && crate.name !== `r-efi-${packageVersion}`) throw Error('r-efi vendor directory does not match its package identity.');
        // Published AUTHORS contains full MIT permission/warranty text plus
        // Apache/LGPL licensing notices. Preserve the reviewed MIT alternative.
        const authors = path.join(directory, 'AUTHORS');
        if (!fs.existsSync(authors) || !fs.lstatSync(authors).isFile() ||
            crypto.createHash('sha256').update(fs.readFileSync(authors)).digest('hex') !== expected)
          throw Error(`r-efi ${packageVersion} requires its reviewed published AUTHORS MIT notice.`);
        authorsNotice = `attributions/dependencies/${crate.name}/AUTHORS`;
      }
      notices(directory, `attributions/dependencies/${crate.name}`, false, authorsNotice);
      // These published archives omit their repository-root license. Add the exact
      // upstream MIT text (or MIT alternative) reviewed in attributions/README.md.
      const supplement = SUPPLEMENTS.find(([name, version]) => packageName === name || [name, `${name}-${version}`].includes(crate.name));
      if (supplement) {
        const [name, version, archive, file, notice] = supplement;
        if (packageName !== name || packageVersion !== version || ![name, `${name}-${version}`].includes(crate.name))
          throw Error(`Unreviewed ${name} package identity for supplemental MIT notice.`);
        const checksum = JSON.parse(fs.readFileSync(path.join(directory, '.cargo-checksum.json'), 'utf8'));
        if (checksum.package !== archive) throw Error(`${name} ${version} requires the exact reviewed archive checksum.`);
        const supplemental = path.join(root, 'attributions', file);
        if (crypto.createHash('sha256').update(fs.readFileSync(supplemental)).digest('hex') !== notice)
          throw Error(`${name} ${version} supplemental MIT notice changed.`);
        copy(supplemental, `attributions/dependencies/${name}-${version}/LICENSE`);
      }
    }
  }
  if(vendor){
    notices(path.join(root,'crates/tsrust-compiler/patched-dependencies/url'),'attributions/dependencies/url-2.5.8');
    copy(path.join(root,'crates/tsrust-compiler/patched-dependencies/url/TSRUST-PATCH.md'),'attributions/dependencies/url-2.5.8/TSRUST-PATCH.md');
  }
  const manifest = {
    schemaVersion: 2, target, version: version('tsrust-compiler'), lspVersion: version('tsrust-lsp'),
    rustToolchain: rust, sourceCommit: git('rev-parse', 'HEAD'),
    sourceTreeDirty: git('status', '--porcelain', '--untracked-files=normal').length > 0,
    distribution: vendor ? 'release' : 'development', files,
  };
  fs.writeFileSync(path.join(destination, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n');
  fs.writeFileSync(path.join(destination, 'rust-toolchain.toml'), `[toolchain]\nchannel = "${rust}"\nprofile = "minimal"\n`);
  fs.writeFileSync(path.join(destination, 'README.md'), `# Tsrust ${manifest.version}\n\nHost: ${target}. Put both executables from bin/ on PATH. Configure the VS Code extension to use this tsrust-lsp. The extension is a separate VSIX release asset.\n\nNative compilation requires Rustup with Rust ${rust} and a system linker; this package does not include Rust itself. Checking, formatting and emission need no external compiler. Prepare backend sources explicitly with tsrust setup --dependencies --yes. Native builds use the shared Cargo cache offline. Portable source exports require explicit tsrust setup --portable --yes preparation; ordinary registry sources are not embedded in these executables. The reviewed URL parser patch is bundled for exact offline materialization.\n\nVerify the archive against the release SHA256SUMS before extraction. manifest.json records consumer and notice hashes, source commit and both consumer versions. Hashes detect corruption, not an untrusted distributor. These executables are not code-signed or notarized.\n\nLicense: MIT; see LICENSE. Third-party notices are in attributions/. Native exported packages retain their own dependency sources and notices. ${vendor ? '' : 'This development package omits dependency notices and is not a redistribution artifact.'}\n\nTsrust is experimental. LSP retention fixes release unrelated editor-buffer snapshots and superseded workspace-scan entries. Bounded Windows and 30-minute macOS probes pass; representative large-workspace and multi-hour memory behavior remains unverified. See the editor troubleshooting guide in the source repository and the release notes for current evidence and limitations.\n`);
  for (const [relative, record] of Object.entries(files)) {
    const digest = crypto.createHash('sha256').update(fs.readFileSync(path.join(destination, relative))).digest('hex');
    if (digest !== record.sha256) throw Error('Copied file digest mismatch.');
  }
  complete = true;
  console.log(destination);
} finally {
  if (!complete) fs.rmSync(destination, {recursive: true, force: true});
}
