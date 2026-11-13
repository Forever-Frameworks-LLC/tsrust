import { Channel, Sender } from 'std:futures';
import {
  Button,
  CodeEditor,
  Node,
  Signal,
  Select,
  Tab,
  Tabs,
  Text,
  Ticker,
  View,
  computed,
  signal,
  translate,
  t
} from 'std:ui';
import { accessibleName, localizeError, message } from '../../lib/localization.tsr';
import { GitRequest, GitReply, DiffHunk, diffHunks } from '../../services/git.tsr';

import { Workspace } from '../../lib/workspace/workspace.tsr';
import { separator } from 'std:path';

export function GitPanel(workspace: Signal<Workspace>, panel: Signal<string>, requests: Sender<GitRequest>, replies: Channel<GitReply>, font: Signal<int>): Node {
  const mode = signal('status');
  const files = signal<string[]>([]);
  const chosen = signal('');
  const selection = computed<string>(() => {
    if (['status', 'working', 'staged'].includes(mode.get()) && chosen.get() !== '') return chosen.get();
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
  const hunks = signal<DiffHunk[]>([]);
  const hunk = signal('');
  const displayed = computed<string>(() => {
    if (shown.get() !== mode.get()) return '';

    if (hunk.get() === '') return output.get();
    const selected = hunks.get().find((entry: ReadOnly<DiffHunk>) => entry.label === hunk.get());
    return selected === undefined ? output.get() : output.get().slice(selected.start, selected.end);
  });
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
          <Tab value="conflicts" label={message('Conflicts')} />
          <Tab value="history" label={message('History')} />
        </Tabs>
        <Button
          icon="plus"
          variant="ghost"
          size="xsmall"
          disabled={busy.get() || pending.get().mode !== '' || selection.get() === '' || hunk.get() !== ''}
          tooltip={message('Stage selected or active file from disk')}
          aria-label={accessibleName('Stage selected or active file from disk')}
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
          disabled={busy.get() || pending.get().mode !== '' || selection.get() === '' || hunk.get() !== ''}
          tooltip={message('Unstage selected or active file')}
          aria-label={accessibleName('Unstage selected or active file')}
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
      <View hidden={mode.get() !== 'status' || shown.get() !== 'status' || busy.get() || files.get().length === 0}>
        <Select
          items={files.get()}
          value={chosen.get()}
          searchable={true}
          cleanable={true}
          size="small"
          placeholder={message('Choose a changed file')}
          accessibilityLabel={message('Choose a changed file')}
          onChange={(path) => {
            chosen.set(clone(path));
          }}
        />
      </View>
      <View hidden={chosen.get() === '' || (mode.get() !== 'working' && mode.get() !== 'staged')}>
        <Text>
          {chosen.get()}
        </Text>
      </View>
      <View hidden={shown.get() !== mode.get() || hunks.get().length === 0}>
        <Select
          items={hunks.get().map((entry: ReadOnly<DiffHunk>) => clone(entry.label))}
          value={hunk.get()}
          searchable={true}
          cleanable={true}
          size="small"
          placeholder={message('Choose a diff hunk')}
          accessibilityLabel={message('Choose a diff hunk')}
          onChange={(label) => {
            hunk.set(clone(label));
          }}
        />
      </View>
      <View hidden={hunk.get() === '' || shown.get() !== mode.get()}>
        <Button
          size="small"
          disabled={busy.get() || pending.get().mode !== ''}
          aria-label={mode.get() === 'working' ? translate('Stage hunk') : translate('Unstage hunk')}
          onPress={() => {
            const selected = hunks.get().find((entry: ReadOnly<DiffHunk>) => entry.label === hunk.get());

            if (selected !== undefined) pending.set({
              mode: mode.get(), action: mode.get() === 'working' ? 'stage-hunk' : 'unstage-hunk', hunk: hunk.get(), patch: selected.header + displayed.get()
            });
          }}
        >
          {mode.get() === 'working' ? t('Stage hunk') : t('Unstage hunk')}
        </Button>
      </View>
      <View hidden={shown.get() === mode.get()}>
        <Text>
          {t('Reading repository…')}
        </Text>
      </View>
      <View hidden={shown.get() !== mode.get() || failed.get() || output.get() !== ''}>
        <Text>
          {mode.get() === 'history' ? t('No commits in this workspace.') : mode.get() === 'conflicts' ? t('No unresolved conflicts.') : t('No changes in this view.')}
        </Text>
      </View>
      <CodeEditor value={displayed.get()} readOnly={true} appearance={false} bordered={false} className={style.get()} accessibilityLabel={message('Source Control')} />
      <Ticker
        interval={150}
        running={panel.get() === 'git' && (shown.get() !== mode.get() || busy.get() || pending.get().mode !== '')}
        onTick={(_frame) => {
          if (busy.get()) {
            match (replies.tryReceive()) {
              Ok(reply) => {
                if (reply.mode === 'status') {
                  const paths = reply.files ?? [];

                  if (!paths.includes(chosen.get())) chosen.set('');
                  files.set(paths);
                }

                hunk.set('');
                hunks.set(!reply.failed && (reply.mode === 'working' || reply.mode === 'staged') ? diffHunks(reply.text) : []);
                output.set(reply.failed ? localizeError(reply.text) : reply.text);
                failed.set(reply.failed);
                shown.set(reply.mode);
                busy.set(false);
              },
              Err(error) => {
                if (error.code !== 'empty') {
                  hunk.set('');
                  hunks.set([]);
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

            if ((request.mode === 'working' || request.mode === 'staged') && chosen.get() !== '') request.viewFile = chosen.get();

            match (requests.trySend(request)) {
              Ok(_) => {
                failed.set(false);
                busy.set(true);
              },
              Err(error) => {
                hunk.set('');
                hunks.set([]);
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
