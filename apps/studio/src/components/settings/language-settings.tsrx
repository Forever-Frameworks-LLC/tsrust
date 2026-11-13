import {
  Node,
  Signal,
  Input,
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  SettingsPage,
  SettingsGroup,
  SettingItem,
  Switch,
  signal,
  translate
} from 'std:ui';

import { languagePresets } from '../../lib/language/presets.tsr';
import { LanguageServer } from '../../lib/language/servers.tsr';
import { message } from '../../lib/localization.tsr';
import { SettingsSession, commitSettings, settingsSessionHandles } from '../../lib/settings/settings-actions.tsr';
import { StudioConfig } from '../../lib/settings/settings.tsr';

function ServerExecutable(session: SettingsSession, server: ReadOnly<LanguageServer>, font: Signal<int>): Node {
  const program = signal(clone(server.program));
  const id = clone(server.id);
  return (
    <SettingItem title={clone(server.name)} description={message('Executable name or absolute path. The server must already be installed.')}>
      <InputGroup size="small" className="w-56 min-w-0">
        <Input
          id={'language-server-' + id}
          value={program.get()}
          aria-label={server.name + ' ' + translate('Executable')}
          className={'text-[' + font.get().toString() + 'px]'}
          onChange={(event) => {
            program.set(clone(event.target.value));
          }}
        />
        <InputGroupAddon align="inline-end">
          <InputGroupButton
            icon="check"
            tooltip={message('Apply executable')}
            accessibilityLabel={message('Apply executable')}
            onClick={() => {
              const config = session.draft.get();
              const index = config.languageServers.findIndex((entry) => entry.id === id);

              if (index >= 0) {
                config.languageServers[index].program = program.get();
                commitSettings(session, config);
              }
            }}
          />
        </InputGroupAddon>
      </InputGroup>
    </SettingItem>
  );
}

function PresetSetting(session: SettingsSession, preset: ReadOnly<LanguageServer>, draft: Signal<StudioConfig>): Node {
  const candidate = clone(preset);
  const id = clone(preset.id);
  const selectedId = clone(id);
  return (
    <SettingItem title={clone(preset.name)} description={message('Enable for matching files. Server changes take effect after restarting Studio.')}>
      <Switch
        checked={draft.get().languageServers.some((server) => server.id === selectedId)}
        accessibilityLabel={clone(preset.name)}
        onChange={(enabled) => {
          const config = session.draft.get();
          config.languageServers = config.languageServers.filter((server) => server.id !== id);

          if (enabled) config.languageServers.push(clone(candidate));
          commitSettings(session, config);
        }}
      />
    </SettingItem>
  );
}

export function LanguageSettings(session: SettingsSession, font: Signal<int>, draft: Signal<StudioConfig>): Node {
  const presetSession = settingsSessionHandles(session);
  const executableSession = settingsSessionHandles(session);
  const renderPreset = (preset: ReadOnly<LanguageServer>): Node => PresetSetting(settingsSessionHandles(presetSession), preset, draft.handle());
  const renderExecutable = (server: ReadOnly<LanguageServer>): Node => ServerExecutable(settingsSessionHandles(executableSession), server, font);
  return (
    <SettingsPage title={message('Language services')} icon="languages" resettable={false}>
      <SettingsGroup title={message('External servers')}>
        {...languagePresets().map(renderPreset)}
      </SettingsGroup>
      <SettingsGroup title={message('Executables')}>
        {...draft.get().languageServers.map(renderExecutable)}
      </SettingsGroup>
    </SettingsPage>
  );
}
