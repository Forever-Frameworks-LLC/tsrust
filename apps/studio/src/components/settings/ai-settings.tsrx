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
  signal
} from 'std:ui';

import { message, accessibleName } from '../../lib/localization.tsr';
import { SettingsSession, commitSettings, settingsSessionHandles } from '../../lib/settings/settings-actions.tsr';
import { StudioConfig } from '../../lib/settings/settings.tsr';

function ArtifactPath(session: SettingsSession, model: boolean, font: Signal<int>): Node {
  const path = signal(model ? session.draft.get().aiModelPath : session.draft.get().aiTokenizerPath);
  return (
    <SettingItem title={model ? message('GGUF model') : message('Tokenizer JSON')} description={message('Absolute path to a compatible local artifact. No automatic downloads or cloud fallback.')}>
      <InputGroup size="small" className="w-56 min-w-0">
        <Input
          value={path.get()}
          aria-label={model ? accessibleName('GGUF model') : accessibleName('Tokenizer JSON')}
          className={'font-mono text-[' + font.get().toString() + 'px]'}
          onChange={(event) => {
            path.set(clone(event.target.value));
          }}
        />
        <InputGroupAddon align="inline-end">
          <InputGroupButton
            icon="check"
            tooltip={message('Apply artifact path')}
            accessibilityLabel={message('Apply artifact path')}
            onClick={() => {
              const config = session.draft.get();

              if (model) config.aiModelPath = path.get();
              else config.aiTokenizerPath = path.get();
              commitSettings(session, config);
            }}
          />
        </InputGroupAddon>
      </InputGroup>
    </SettingItem>
  );
}

export function AiSettings(session: ReadOnly<SettingsSession>, font: Signal<int>): Node {
  return (
    <SettingsPage title={message('AI')} icon="sparkles" resettable={false}>
      <SettingsGroup title={message('Local model')}>
        {ArtifactPath(settingsSessionHandles(session), true, font)}
        {ArtifactPath(settingsSessionHandles(session), false, font)}
      </SettingsGroup>
    </SettingsPage>
  );
}
