import path from 'node:path';

// Cargo resolves a relative CARGO_TARGET_DIR against its invocation directory.
export function compilerExecutable(root, cwd, env = process.env, platform = process.platform) {
 const target = env.CARGO_TARGET_DIR ? path.resolve(cwd, env.CARGO_TARGET_DIR) : path.join(root, 'target');
 return path.join(target, 'release', platform === 'win32' ? 'tsrust.exe' : 'tsrust');
}
