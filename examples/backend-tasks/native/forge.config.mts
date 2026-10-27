import type { ForgeConfig } from '@electron-forge/shared-types';
import { MakerZIP } from '@electron-forge/maker-zip';
import { VitePlugin } from '@electron-forge/plugin-vite';
import { FusesPlugin } from '@electron-forge/plugin-fuses';
import { FuseV1Options, FuseVersion } from '@electron/fuses';
import path from 'node:path';

const config: ForgeConfig = {
  packagerConfig: { asar: true, executableName: 'tsrust-tasks',
    extraResource: [path.resolve('resources/backend')],
  },
  rebuildConfig: {},
  makers: [new MakerZIP({}, ['win32'])],
  plugins: [new VitePlugin({ concurrent: 1, build: [
    { entry: 'src/main.ts', config: 'vite.main.config.mts', target: 'main' },
    { entry: 'src/preload.ts', config: 'vite.preload.config.mts', target: 'preload' },
  ], renderer: [{ name: 'main_window', config: 'vite.renderer.config.mts' }] }),
  new FusesPlugin({ version: FuseVersion.V1,
    [FuseV1Options.RunAsNode]: false, [FuseV1Options.EnableCookieEncryption]: true,
    [FuseV1Options.EnableNodeOptionsEnvironmentVariable]: false, [FuseV1Options.EnableNodeCliInspectArguments]: false,
    [FuseV1Options.EnableEmbeddedAsarIntegrityValidation]: true, [FuseV1Options.OnlyLoadAppFromAsar]: true,
    [FuseV1Options.GrantFileProtocolExtraPrivileges]: false,
  })],
};
export default config;
