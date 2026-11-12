import { Channel } from 'std:futures';
import {
  View,
  Text,
  Button,
  Textarea,
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  Message,
  MessageScroller,
  Markdown,
  Node,
  Signal,
  signal,
  computed,
  translate,
  t
} from 'std:ui';

import { StudioConfig } from '../../lib/settings/settings.tsr';
import { Workspace } from '../../lib/workspace/workspace.tsr';
import { accessibleName, message, localizeError } from '../../lib/localization.tsr';
import { showSettings } from '../settings/settings-window.tsrx';
import { AiState, AiContext, invokeModelTool } from '../../services/ai/actions.tsr';
import { ModelRequest } from '../../lib/ai/protocol.tsr';

function context(state: ReadOnly<Signal<AiState>>, settings: ReadOnly<Signal<StudioConfig>>, requests: ReadOnly<Channel<ModelRequest>>): AiContext {
  return {
    state: state.handle(), settings: settings.handle(), commands: requests.sender(), allowRead: true, allowControl: true, allowInference: true
  };
}

function perform(authority: ReadOnly<AiContext>, name: string, payload: string = '{}'): boolean {
  match (invokeModelTool(authority, name, payload)) {
    Ok(_) => {
      return true;
    },
    Err(error) => {
      const next = authority.state.get();
      next.notice = error;
      authority.state.set(next);
      return false;
    }
  }
}

function send(authority: ReadOnly<AiContext>, prompt: Signal<string>): void {
  if (perform(authority, 'local_model_send', JSON.stringify({
    prompt: prompt.get()
  }))) prompt.set('');
}

export function ChatPanel(
  open: Signal<boolean>,
  state: Signal<AiState>,
  settings: Signal<StudioConfig>,
  requests: ReadOnly<Channel<ModelRequest>>,
  font: Signal<int>,
  workspace: Signal<Workspace>
): Node {
  const prompt = signal('');
  const style = computed<string>(() => 'flex-col size-full min-w-0 min-h-0 bg-background text-[' + font.get().toString() + 'px]');
  const inputStyle = computed<string>(() => 'min-w-0 font-mono text-[' + font.get().toString() + 'px]');
  const lifecycle = context(state, settings, requests);
  const clear = context(state, settings, requests);
  const submit = context(state, settings, requests);
  const composer = context(state, settings, requests);
  return (
    <View className={style.get()}>
      <View className="flex-row items-center gap-1 px-2 py-1 shrink-0 border-b border-border">
        <Text className="grow">
          {t('AI Chat')}
        </Text>
        <Button
          icon="x"
          size="xsmall"
          variant="ghost"
          tooltip={message('Close AI Chat')}
          aria-label={accessibleName('Close AI Chat')}
          onPress={() => {
            open.set(false);
          }}
        />
      </View>
      <View className="shrink-0 px-3 py-2">
        <Text>
          {t('Experimental local inference · 32 output tokens · no tools')}
        </Text>
        <Text>
          {translate(state.get().phase)}
        </Text>
        <View hidden={state.get().notice === ''}>
          <Text>
            {localizeError(state.get().notice)}
          </Text>
        </View>
      </View>
      <MessageScroller className="grow min-h-0" contentClassName="p-3" rowClassName="py-2">
        {...state.get().messages.map((entry) => (
          <Message author={entry.role === 'user' ? translate('You') : translate('Local model')} variant="ghost" align={entry.role === 'user' ? 'end' : 'start'}>
            <Markdown content={clone(entry.text)} />
          </Message>
        ))}
      </MessageScroller>
      <View className="shrink-0 min-w-0 p-2 flex-col">
        <InputGroup size="small" className={inputStyle.get()} accessibilityLabel={message('AI composer')}>
          <Textarea
            value={prompt.get()}
            placeholder={message('Ask the local model')}
            aria-label={accessibleName('Ask the local model')}
            className={inputStyle.get()}
            minRows={2}
            maxRows={8}
            submitOnEnter={true}
            onChange={(event) => {
              prompt.set(clone(event.target.value));
            }}
            onSubmit={(_secondary) => {
              if (state.get().phase === 'ready') send(submit, prompt);
            }}
          />
          <InputGroupAddon align="block-end" className="min-w-0 gap-1">
            <InputGroupButton
              icon={state.get().phase === 'stopped' ? 'play' : 'square'}
              disabled={state.get().phase === 'stopping'}
              tooltip={translate(state.get().phase === 'stopped' ? 'Start local model' : 'Stop local model')}
              accessibilityLabel={translate(state.get().phase === 'stopped' ? 'Start local model' : 'Stop local model')}
              onClick={() => {
                perform(lifecycle, state.get().phase === 'stopped' ? 'local_model_start' : 'local_model_stop');
              }}
            />
            <InputGroupButton
              icon="trash"
              disabled={state.get().phase !== 'ready' && state.get().phase !== 'stopped'}
              tooltip={translate('Clear conversation')}
              accessibilityLabel={translate('Clear conversation')}
              onClick={() => {
                perform(clear, 'local_model_clear');
              }}
            />
            <InputGroupButton
              icon="settings"
              tooltip={translate('AI settings')}
              accessibilityLabel={translate('AI settings')}
              onClick={() => {
                showSettings(font, settings, workspace);
              }}
            />
            <View className="grow min-w-0" />
            <InputGroupButton
              icon={state.get().phase === 'generating' ? 'square' : 'arrow-up'}
              disabled={(state.get().phase !== 'ready' && state.get().phase !== 'generating') || (state.get().phase === 'ready' && prompt.get().trim() === '')}
              tooltip={translate(state.get().phase === 'generating' ? 'Cancel response' : 'Send prompt')}
              accessibilityLabel={translate(state.get().phase === 'generating' ? 'Cancel response' : 'Send prompt')}
              onClick={() => {
                if (state.get().phase === 'generating') perform(composer, 'local_model_cancel', JSON.stringify({
                  request: state.get().active
                }));
                else send(composer, prompt);
              }}
            />
          </InputGroupAddon>
        </InputGroup>
      </View>
    </View>
  );
}
