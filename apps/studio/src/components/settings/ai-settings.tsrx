import { toInt } from 'std:numeric';
import {
  Node,
  NumberInput,
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
  const budgetSession = settingsSessionHandles(session);
  return (
    <SettingsPage title={message('AI')} icon="sparkles" resettable={false}>
      <SettingsGroup title={message('Local model')}>
        {ArtifactPath(settingsSessionHandles(session), true, font)}
        {ArtifactPath(settingsSessionHandles(session), false, font)}
        <SettingItem title={message('Response token budget')} description={message('Maximum tokens per response (1–4096). New requests use this budget; context, output-byte and time limits still apply.')}>
          <NumberInput
            value={Number(session.draft.get().aiResponseTokens)}
            min={1}
            max={4096}
            step={1}
            size="small"
            className="w-28"
            accessibilityLabel={message('Response token budget')}
            onValidate={(candidate) => Number.isInteger(Number(candidate))}
            onChange={(value) => {
              match (toInt(value)) {
                Ok(tokens) => {
                  const config = budgetSession.draft.get();
                  config.aiResponseTokens = tokens;
                  commitSettings(budgetSession, config);
                },
                Err(_) => {}
              }
            }}
          />
        </SettingItem>
      </SettingsGroup>
    </SettingsPage>
  );
}
