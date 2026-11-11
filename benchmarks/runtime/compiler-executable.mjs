import path from 'node:path';
import fs from 'node:fs';

// Cargo resolves a relative CARGO_TARGET_DIR against its invocation directory.
export function compilerExecutable(root, cwd, env = process.env, platform = process.platform) {
 if (env.TSRUST) {
  const executable=path.resolve(cwd,env.TSRUST);
  if (!fs.statSync(executable).isFile()) throw new Error(`TSRUST must name a compiler executable: ${executable}`);
  fs.accessSync(executable,fs.constants.X_OK);
  return executable;
 }
 const target = env.CARGO_TARGET_DIR ? path.resolve(cwd, env.CARGO_TARGET_DIR) : path.join(root, 'target');
 return path.join(target, 'release', platform === 'win32' ? 'tsrust.exe' : 'tsrust');
}
