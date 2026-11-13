import { Channel, Sender } from 'std:futures';
import {
  Button,
  CodeEditor,
  Node,
  Signal,
  Tab,
  Tabs,
  Text,
  Ticker,
  View,
  computed,
  signal,
  t
} from 'std:ui';
import { accessibleName, message } from '../../lib/localization.tsr';
import { GitRequest, GitReply } from '../../services/git.tsr';

import { Workspace } from '../../lib/workspace/workspace.tsr';
import { separator } from 'std:path';

export function GitPanel(workspace: Signal<Workspace>, panel: Signal<string>, requests: Sender<GitRequest>, replies: Channel<GitReply>, font: Signal<int>): Node {
  const mode = signal('status');
  const selection = computed<string>(() => {
    const state = workspace.get();
    const prefix = state.root.endsWith(separator()) ? clone(state.root) : state.root + separator();
    return state.active.startsWith(prefix) ? state.active.slice(prefix.length) : '';
  });
  const pending = signal<GitRequest>({
    mode: ''
  });
  const shown = signal('');
  const busy = signal(false);
  const output = signal('');
  const failed = signal(false);
  const style = computed<string>(() => 'grow min-h-0 font-mono text-[' + font.get().toString() + 'px]');
  return (
    <View className="flex-col size-full min-h-0">
      <View className="flex-row items-center">
        <Tabs
          size="small"
          value={mode.get()}
          onChange={(value) => {
            mode.set(clone(value));
          }}
        >
          <Tab value="status" label={message('Status')} />
          <Tab value="working" label={message('Working diff')} />
          <Tab value="staged" label={message('Staged diff')} />
        </Tabs>
        <Button
          icon="plus"
          variant="ghost"
          size="xsmall"
          disabled={busy.get() || pending.get().mode !== '' || selection.get() === ''}
          tooltip={message('Stage active file from disk')}
          aria-label={accessibleName('Stage active file from disk')}
          onPress={() => {
            pending.set({
              mode: mode.get(), action: 'stage', path: selection.get()
            });
          }}
        />
        <Button
          icon="minus"
          variant="ghost"
          size="xsmall"
          disabled={busy.get() || pending.get().mode !== '' || selection.get() === ''}
          tooltip={message('Unstage active file')}
          aria-label={accessibleName('Unstage active file')}
          onPress={() => {
            pending.set({
              mode: mode.get(), action: 'unstage', path: selection.get()
            });
          }}
        />
        <Button
          icon="refresh-cw"
          variant="ghost"
          size="xsmall"
          disabled={busy.get() || pending.get().mode !== ''}
          tooltip={message('Refresh source control')}
          aria-label={accessibleName('Refresh source control')}
          onPress={() => {
            shown.set('');
          }}
        />
      </View>
      <View hidden={shown.get() === mode.get()}>
        <Text>
          {t('Reading repository…')}
        </Text>
      </View>
      <View hidden={shown.get() !== mode.get() || failed.get() || output.get() !== ''}>
        <Text>
          {t('No changes in this view.')}
        </Text>
      </View>
      <CodeEditor value={shown.get() === mode.get() ? output.get() : ''} readOnly={true} appearance={false} bordered={false} className={style.get()} accessibilityLabel={message('Source Control')} />
      <Ticker
        interval={150}
        running={panel.get() === 'git' && (shown.get() !== mode.get() || busy.get() || pending.get().mode !== '')}
        onTick={(_frame) => {
          if (busy.get()) {
            match (replies.tryReceive()) {
              Ok(reply) => {
                output.set(reply.text);
                failed.set(reply.failed);
                shown.set(reply.mode);
                busy.set(false);
              },
              Err(error) => {
                if (error.code !== 'empty') {
                  output.set(clone(error.code));
                  failed.set(true);
                  shown.set(clone(mode.get()));
                  busy.set(false);
                }
              }
            }
          } else {
            const request = pending.get();
            pending.set({
              mode: ''
            });

            if (request.mode === '') request.mode = mode.get();

            match (requests.trySend(request)) {
              Ok(_) => {
                failed.set(false);
                busy.set(true);
              },
              Err(error) => {
                output.set(clone(error.code));
                failed.set(true);
                shown.set(clone(mode.get()));
              }
            }
          }
        }}
      />
    </View>
  );
}
