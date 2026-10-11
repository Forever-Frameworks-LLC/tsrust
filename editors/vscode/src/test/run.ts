import * as path from 'node:path';
import * as fs from 'node:fs';
import * as os from 'node:os';
import { runTests, resolveCliArgsFromVSCodeExecutablePath } from '@vscode/test-electron';
import { execFileSync } from 'node:child_process';

async function main(): Promise<void> {
  const developmentPath = path.resolve(__dirname, '../..');
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'tsrust-vscode-test-'));
  const workspace = path.join(temporary, 'workspace');
  fs.mkdirSync(path.join(workspace, '.vscode'), { recursive: true });
  const serverPath = process.env.TSRUST_LSP_PATH || path.resolve(developmentPath, '../../target/editor/debug/tsrust-lsp');
  fs.writeFileSync(path.join(workspace, '.vscode/settings.json'), JSON.stringify({
    ...(process.env.TSRUST_TEST_AUTO_DISCOVERY ? {} : {'tsrust.server.path': serverPath}), 'tsrust.analysis.debounceMs': 20,
    'security.workspace.trust.enabled': false,
  }));
  try {
    let testedPath: string | string[] = developmentPath;
    if (process.env.TSRUST_VSIX_PATH) {
      if (!process.env.VSCODE_EXECUTABLE) throw new Error('Packaged tests require VSCODE_EXECUTABLE');
      const [cli, ...args] = resolveCliArgsFromVSCodeExecutablePath(process.env.VSCODE_EXECUTABLE, { reuseMachineInstall: true });
      execFileSync(cli, [...args, '--user-data-dir', path.join(temporary, 'user'), '--extensions-dir', path.join(temporary, 'extensions'), '--install-extension', process.env.TSRUST_VSIX_PATH, '--force'], { stdio: 'inherit' });
      // The host needs a development path to run tests; point it at the exact
      // installed package, not the source checkout.
      const installed = fs.readdirSync(path.join(temporary, 'extensions')).find(name => name.startsWith('tsrust-local.tsrust-'));
      if (!installed) throw new Error('Installed VSIX was not found');
      testedPath = path.join(temporary, 'extensions', installed);
    }
    await runTests({
      vscodeExecutablePath: process.env.VSCODE_EXECUTABLE,
      extensionDevelopmentPath: testedPath,
      extensionTestsPath: path.join(__dirname, 'suite'),
      launchArgs: [workspace, '--user-data-dir', path.join(temporary, 'user'), '--extensions-dir', path.join(temporary, 'extensions'), '--disable-workspace-trust', '--skip-welcome', '--skip-release-notes'],
    });
  } finally { fs.rmSync(temporary, { recursive: true, force: true }); }
}
main().catch(error => { console.error(error); process.exitCode = 1; });
