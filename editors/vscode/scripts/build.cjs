const fs = require('node:fs');
const path = require('node:path');
fs.copyFileSync(path.resolve(__dirname, '../../../LICENSE'), 'LICENSE');
const result = require('esbuild').buildSync({
  entryPoints: ['src/extension.ts'], bundle: true, platform: 'node',
  target: 'node20', format: 'cjs', outfile: 'out/extension.js', external: ['vscode'],
  metafile: true,
});
// The language client invokes this helper if a server fails to shut down.
fs.copyFileSync(require.resolve('vscode-languageclient/lib/node/terminateProcess.sh'), 'out/terminateProcess.sh');
const packages = new Set();
for (const input of Object.keys(result.metafile.inputs)) {
  if (!input.includes('node_modules/')) continue;
  let directory = path.dirname(path.resolve(input));
  while (!fs.existsSync(path.join(directory, 'package.json'))) {
    const parent = path.dirname(directory);
    if (parent === directory) throw new Error(`No package license for ${input}`);
    directory = parent;
  }
  packages.add(directory);
}
const notices = [...packages].sort().map(directory => {
  const pkg = JSON.parse(fs.readFileSync(path.join(directory, 'package.json'), 'utf8'));
  const license = fs.readdirSync(directory).find(name => /^licen[sc]e(?:\.|$)/i.test(name));
  if (!license) throw new Error(`No license text for ${pkg.name}`);
  return `${pkg.name} ${pkg.version} (${pkg.license})\n\n${fs.readFileSync(path.join(directory, license), 'utf8')}`;
});
fs.writeFileSync('THIRD_PARTY_NOTICES.txt', notices.join('\n\n----------------------------------------\n\n'));
