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
  const serverPath = process.env.TSRUST_LSP_PATH || path.resolve(developmentPath, '../../target/debug/tsrust-lsp' + (process.platform === 'win32' ? '.exe' : ''));
  fs.writeFileSync(path.join(workspace, '.vscode/settings.json'), JSON.stringify({
    ...(process.env.TSRUST_TEST_AUTO_DISCOVERY ? {} : {'tsrust.server.path': serverPath}), 'tsrust.analysis.debounceMs': 20,
    'security.workspace.trust.enabled': false,
  }));
  try {
    let testedPath: string | string[] = developmentPath;
    if (process.env.TSRUST_VSIX_PATH) {
      if (!process.env.VSCODE_EXECUTABLE) throw new Error('Packaged tests require VSCODE_EXECUTABLE');
      const [cli, ...args] = resolveCliArgsFromVSCodeExecutablePath(process.env.VSCODE_EXECUTABLE, { reuseMachineInstall: true });
      const installArgs = [...args, '--user-data-dir', path.join(temporary, 'user'), '--extensions-dir', path.join(temporary, 'extensions'), '--install-extension', process.env.TSRUST_VSIX_PATH, '--force'];
      if (process.platform === 'win32') {
        // Invoke Electron's CLI entry directly so paths never pass through cmd.exe.
        const executable = process.env.VSCODE_EXECUTABLE;
        // Installed VS Code can keep resources in a versioned sibling directory.
        const entry = fs.readFileSync(cli, 'utf8').match(/"(%~dp0[^"\r\n]*[\\/]cli\.js)"/i)?.[1];
        if (!entry) throw new Error(`Cannot locate the Windows VS Code CLI entry in ${cli}`);
        const script = path.resolve(path.dirname(cli), entry.slice('%~dp0'.length));
        execFileSync(executable, [script, ...installArgs], {stdio: 'inherit', env: {...process.env, ELECTRON_RUN_AS_NODE: '1'}});
      } else {
        execFileSync(cli, installArgs, {stdio: 'inherit'});
      }
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
  } finally {
    if (path.dirname(path.resolve(temporary)) !== path.resolve(os.tmpdir()) || !path.basename(temporary).startsWith('tsrust-vscode-test-')) throw new Error('Unexpected test cleanup directory');
    fs.rmSync(temporary, { recursive: true, force: true });
  }
}
main().catch(error => { console.error(error); process.exitCode = 1; });
