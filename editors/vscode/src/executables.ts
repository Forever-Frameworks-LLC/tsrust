import {constants} from 'node:fs';
import {access,stat} from 'node:fs/promises';
import {homedir} from 'node:os';
import {delimiter,dirname,isAbsolute,join} from 'node:path';

export async function resolveExecutable(name: string, options: {
  configured?: string; companion?: string; home?: string; envPath?: string;
} = {}): Promise<string> {
  const executable = process.platform === 'win32' ? `${name}.exe` : name;
  const verify = async (file: string) => {
    if (!isAbsolute(file)) throw new Error(`Executable path must be absolute: ${file}`);
    if (!(await stat(file)).isFile()) throw new Error(`Not an executable file: ${file}`);
    await access(file, constants.X_OK);
    return file;
  };
  if (options.configured) return verify(options.configured);
  const candidates = [
    ...(options.companion && isAbsolute(options.companion) ? [join(dirname(options.companion), executable)] : []),
    join(options.home ?? homedir(), '.cargo', 'bin', executable),
    ...(options.envPath ?? process.env.PATH ?? '').split(delimiter).filter(isAbsolute).map(directory => join(directory, executable)),
  ];
  for (const candidate of new Set(candidates)) {
    try { return await verify(candidate); } catch { /* Try the next installed location. */ }
  }
  throw new Error(`Cannot find ${executable}. Install it in ~/.cargo/bin or PATH, or configure its absolute path in Tsrust settings.`);
}
