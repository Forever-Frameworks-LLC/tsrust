import { FormattingTarget } from '../../lib/workspace/formatting-actions.tsr';
import { Channel, Sender } from 'std:futures';
import { join, extname } from 'std:path';
import {
  Button,
  CodeEditor,
  Diff,
  Input,
  List,
  ListItem,
  ListItemSuffix,
  Node,
  Select,
  Signal,
  Text,
  Ticker,
  View,
  computed,
  signal,
  t,
  translate
} from 'std:ui';
import { accessibleName, message, localizeError } from '../../lib/localization.tsr';
import { DiffHunk, GitChange, GitReply, GitRequest, diffHunks } from '../../services/git.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { openFile, OpenFile, Workspace } from '../../lib/workspace/workspace.tsr';
import { publishWorkspace, saveTarget, updateDraft } from '../../lib/workspace/document-actions.tsr';
import { resolveFirstConflict } from '../../lib/workspace/merge.tsr';

function ChangeRow(row: ReadOnly<GitChange>, pending: Signal<GitRequest>, busy: Signal<boolean>): Node {
  const file = clone(row.path);
  const staged = row.mode === 'staged';
  return (
    <ListItem value={row.mode + '/' + row.path} accessibilityLabel={clone(row.path)}>
      <Text className="grow truncate">
        {clone(row.path)}
      </Text>
      <Text>
        {clone(row.status)}
      </Text>
      <ListItemSuffix>
        <View hidden={row.mode === 'conflicts'}>
          <Button
            icon={staged ? 'minus' : 'plus'}
            size="xsmall"
            variant="ghost"
            disabled={busy.get() || pending.get().mode !== ''}
            aria-label={staged ? translate('Unstage file') : translate('Stage file')}
            onPress={() => {
              pending.set({
                mode: 'status', action: staged ? 'unstage' : 'stage', path: clone(file)
              });
            }}
          />
        </View>
      </ListItemSuffix>
    </ListItem>
  );
}

function pickChange(
  key: string,
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>,
  viewing: Signal<boolean>,
  changes: Signal<GitChange[]>,
  selected: Signal<GitChange>,
  errorText: Signal<string>,
  output: Signal<string>,
  hunks: Signal<DiffHunk[]>,
  hunk: Signal<string>,
  pending: Signal<GitRequest>,
  busy: Signal<boolean>
): void {
  if (busy.get() || pending.get().mode !== '') return;
  const row = changes.get().find((item: ReadOnly<GitChange>) => item.mode + '/' + item.path === key);

  if (row === undefined) return;
  selected.set(clone(row));
  output.set('');
  hunks.set([]);
  hunk.set('');
  errorText.set('');

  if (row.mode === 'conflicts') {
    const next = workspace.get();

    const filePath = join(next.root, row.path);

    match (openFile(next, filePath)) {
      Err(reason) => {
        errorText.set(reason);
      },
      Ok(_) => {}
    }

    publishWorkspace(next, workspace, recovery);
  }

  pending.set({
    mode: clone(row.mode), viewFile: clone(row.path)
  });
  viewing.set(true);
}

function ChangeGroup(
  kind: string,
  title: string,
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>,
  viewing: Signal<boolean>,
  changes: Signal<GitChange[]>,
  selected: Signal<GitChange>,
  errorText: Signal<string>,
  output: Signal<string>,
  hunks: Signal<DiffHunk[]>,
  hunk: Signal<string>,
  pending: Signal<GitRequest>,
  busy: Signal<boolean>
): Node {
  const groupKind = clone(kind);
  return (
    <View hidden={!changes.get().some((item: ReadOnly<GitChange>) => item.mode === kind)} className="flex-col grow min-h-0">
      <Text className="px-2 py-1 font-semibold">
        {title}
      </Text>
      <List
        className="grow min-h-0"
        size="small"
        onConfirm={(key) => {
          pickChange(
            clone(key),
            workspace.handle(),
            recovery.handle(),
            viewing.handle(),
            changes.handle(),
            selected.handle(),
            errorText.handle(),
            output.handle(),
            hunks.handle(),
            hunk.handle(),
            pending.handle(),
            busy.handle()
          );
        }}
      >
        {...changes.get().filter((item: ReadOnly<GitChange>) => item.mode === groupKind).map((row: ReadOnly<GitChange>) => ChangeRow(row, pending.handle(), busy.handle()))}
      </List>
    </View>
  );
}

export function GitPanel(
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>,
  active: Signal<string>,
  visible: Signal<boolean>,
  viewing: Signal<boolean>,
  requests: Sender<GitRequest>,
  replies: Channel<GitReply>,
  selected: Signal<GitChange>,
  errorText: Signal<string>,
  output: Signal<string>,
  ours: Signal<string>,
  theirs: Signal<string>,
  busy: Signal<boolean>,
  pending: Signal<GitRequest>,
  hunk: Signal<string>,
  hunks: Signal<DiffHunk[]>
): Node {
  const changes = signal<GitChange[]>([]);
  const messageText = signal('');
  const loaded = signal(false);
  const sent = signal<GitRequest>({
    mode: ''
  });
  const commits = signal<string[]>([]);
  const history = signal(false);
  const checkingBusy = busy.handle();
  const checkingPending = pending.handle();
  const locked = computed<boolean>(() => checkingBusy.get() || checkingPending.get().mode !== '');
  const sidebar: Node = <View className="size-full flex-col min-h-0">
    <View className="flex-row items-center px-2">
      <Text className="grow">
        {t('Source Control')}
      </Text>
      <Button
        icon="history"
        variant="ghost"
        size="xsmall"
        aria-label={accessibleName('History')}
        disabled={locked.get()}
        onPress={() => {
          history.set(!history.get());

          if (history.get()) pending.set({
            mode: 'history'
          });
        }}
      />
      <Button
        icon="refresh-cw"
        variant="ghost"
        size="xsmall"
        aria-label={accessibleName('Refresh source control')}
        disabled={locked.get()}
        onPress={() => {
          pending.set({
            mode: 'status'
          });
        }}
      />
    </View>
    <Input
      value={messageText.get()}
      placeholder={message('Commit message')}
      aria-label={accessibleName('Commit message')}
      onChange={(event) => {
        messageText.set(clone(event.target.value));
      }}
    />
    <Button
      size="small"
      disabled={locked.get() || messageText.get().trim() === '' || !changes.get().some((row: ReadOnly<GitChange>) => row.mode === 'staged')}
      onPress={() => {
        pending.set({
          mode: 'status', action: 'commit-staged', message: messageText.get()
        });
      }}
    >
      {t('Commit staged changes')}
    </Button>
    <View hidden={!history.get()}>
      <Select
        items={commits.get()}
        value={selected.get().mode === 'commit' ? selected.get().path : ''}
        placeholder={message('Choose a commit')}
        onChange={(value) => {
          if (!locked.get() && value !== '') {
            selected.set({
              path: clone(value), mode: 'commit', status: ''
            });
            output.set('');
            hunk.set('');
            viewing.set(true);
            pending.set({
              mode: 'commit', commit: value.split(' ')[0]
            });
          }
        }}
      />
    </View>
    <View hidden={errorText.get() === ''}>
      <Text className="px-2 text-destructive">
        {errorText.get()}
      </Text>
    </View>
    <View hidden={!loaded.get() || changes.get().length > 0 || errorText.get() !== ''}>
      <Text>
        {t('No changes in this view.')}
      </Text>
    </View>
    {ChangeGroup(
      'conflicts',
      translate('Merge changes'),
      workspace.handle(),
      recovery.handle(),
      viewing.handle(),
      changes.handle(),
      selected.handle(),
      errorText.handle(),
      output.handle(),
      hunks.handle(),
      hunk.handle(),
      pending.handle(),
      busy.handle()
    )}
    {ChangeGroup(
      'staged',
      translate('Staged changes'),
      workspace.handle(),
      recovery.handle(),
      viewing.handle(),
      changes.handle(),
      selected.handle(),
      errorText.handle(),
      output.handle(),
      hunks.handle(),
      hunk.handle(),
      pending.handle(),
      busy.handle()
    )}
    {ChangeGroup(
      'working',
      translate('Changes'),
      workspace.handle(),
      recovery.handle(),
      viewing.handle(),
      changes.handle(),
      selected.handle(),
      errorText.handle(),
      output.handle(),
      hunks.handle(),
      hunk.handle(),
      pending.handle(),
      busy.handle()
    )}
    <Ticker interval={150} running={busy.get() || (active.get() === 'git' && visible.get() && pending.get().mode !== '')} onTick={(_frame) => {
      if (busy.get()) {
        match (replies.tryReceive()) {
          Ok(reply) => {
            busy.set(false);
            const req = sent.get();
            if (reply.failed) { errorText.set(localizeError(reply.text)); return; }
            errorText.set('');
            if (req.action === 'commit-staged') messageText.set('');
            if (reply.mode === 'status') {
              const rows = reply.changes ?? [];
              changes.set(clone(rows)); loaded.set(true);
              const current = selected.get();
              if (current.mode !== 'commit' && current.path !== '') {
                let index = rows.findIndex((row: ReadOnly<GitChange>) => row.path === current.path && row.mode === current.mode);
                if (index < 0) index = rows.findIndex((row: ReadOnly<GitChange>) => row.path === current.path);
                if (index >= 0) {selected.set(clone(rows[index]));pending.set({mode:clone(rows[index].mode),viewFile:clone(rows[index].path)});}
                else {viewing.set(false);selected.set({mode:'',path:'',status:''});}
              }
            } else if (reply.mode === 'history') commits.set(reply.text.split('\n').filter((line) => line !== ''));
            else {ours.set(reply.ours ?? '');theirs.set(reply.theirs ?? '');hunks.set(diffHunks(reply.text));output.set(clone(reply.text));hunk.set('');if (req.action !== undefined) pending.set({mode:'status'});}
          }, Err(reason) => {if (reason.code !== 'empty') {busy.set(false); errorText.set(reason.code);}}
        }
      } else {
        const req = pending.get();
        if (req.mode === '') return;
        match (requests.trySend(clone(req))) { Ok(_) => {busy.set(true);sent.set(req);pending.set({mode:''});}, Err(reason) => {if (reason.code !== 'full') {errorText.set(reason.code);pending.set({mode:''});}} }
      }
    }} />
  </View>;
  return sidebar;
}

function resolveDraft(
  side: string,
  text: string,
  path: string,
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>,
  errorText: Signal<string>
): void {
  match (resolveFirstConflict(text, side)) {
    Ok(text) => {
      updateDraft(workspace, recovery, path, text);
      errorText.set('');
    },
    Err(reason) => {
      errorText.set(reason);
    }
  }
}

export function GitEditor(
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>,
  viewing: Signal<boolean>,
  font: Signal<int>,
  selected: Signal<GitChange>,
  pending: Signal<GitRequest>,
  output: Signal<string>,
  ours: Signal<string>,
  theirs: Signal<string>,
  busy: Signal<boolean>,
  errorText: Signal<string>,
  hunks: Signal<DiffHunk[]>,
  hunk: Signal<string>
): Node {
  const pathSelection = selected.handle();
  const path = computed<string>(() => join(workspace.get().root, pathSelection.get().path));
  const draft = computed<string>(() => {
    const file = workspace.get().files.find((file: ReadOnly<OpenFile>) => file.document.path === path.get() && file.kind === 'text');
    return file === undefined ? '' : clone(file.document.text);
  });
  const checkingBusy = busy.handle();
  const checkingPending = pending.handle();
  const locked = computed<boolean>(() => checkingBusy.get() || checkingPending.get().mode !== '');
  const editor: Node = <View className="size-full flex-col min-h-0">
    <View className="flex-row items-center px-2">
      <Text className="grow truncate">
        {selected.get().path}
      </Text>
      <View hidden={selected.get().mode === 'commit'}>
        <Button size="small" disabled={locked.get()} onPress={() => {
        if (selected.get().mode === 'conflicts') {
          const next = workspace.get();
          const index = next.files.findIndex((entry: ReadOnly<OpenFile>) => entry.document.path === path.get() && entry.kind === 'text');
          if (index >= 0) {
            if (next.files[index].document.text.split('\n').some((line) => line.startsWith('<<<<<<<'))) {errorText.set(translate('Resolve conflict markers before staging.'));return;}
            const target: FormattingTarget = {path:clone(next.files[index].document.path),session:clone(next.files[index].document.session),revision:next.files[index].document.revision};
            match (saveTarget(next,target)) {
              Err(reason) => {errorText.set(reason);publishWorkspace(next,workspace,recovery);return;}, Ok(_) => {}
            }
            publishWorkspace(next,workspace,recovery);
          }
        }
        pending.set({mode:'status',action:selected.get().mode === 'staged' ? 'unstage' : 'stage',path:selected.get().path});
      }}>{selected.get().mode === 'staged' ? t('Unstage file') : selected.get().mode === 'conflicts' ? t('Stage resolution') : t('Stage file')}</Button>
      </View>
      <Button
        icon="x"
        size="xsmall"
        variant="ghost"
        aria-label={accessibleName('Close diff')}
        onPress={() => {
          viewing.set(false);
        }}
      />
    </View>
    <View hidden={hunks.get().length === 0 || !['working', 'staged'].includes(selected.get().mode)} className="flex-row">
      <Select
        items={hunks.get().map((item: ReadOnly<DiffHunk>) => clone(item.label))}
        value={hunk.get()}
        placeholder={message('Choose a diff hunk')}
        onChange={(value) => {
          hunk.set(clone(value));
        }}
      />
      <Button
        size="small"
        disabled={locked.get() || hunk.get() === ''}
        onPress={() => {
          const item = hunks.get().find((entry: ReadOnly<DiffHunk>) => entry.label === hunk.get());

          if (item !== undefined) pending.set({
            mode: selected.get().mode, action: selected.get().mode === 'staged' ? 'unstage-hunk' : 'stage-hunk', viewFile: selected.get().path, hunk: hunk.get(), patch: item.header + output.get().slice(item.start, item.end)
          });
        }}
      >
        {selected.get().mode === 'staged' ? t('Unstage hunk') : t('Stage hunk')}
      </Button>
    </View>
    <View hidden={selected.get().mode !== 'conflicts'} className="flex-col grow min-h-0">
      <View className="grow min-h-0">
        <Diff originalText={ours.get()} modifiedText={theirs.get()} filePath={selected.get().path} mode="split" syntaxHighlight={true} fontSize={font.get()} className="size-full" />
      </View>
      <View className="flex-row">
        <Button
          size="small"
          onPress={() => {
            resolveDraft(
              'current',
              draft.get(),
              path.get(),
              workspace.handle(),
              recovery.handle(),
              errorText.handle()
            );
          }}
        >
          {t('Accept current')}
        </Button>
        <Button
          size="small"
          onPress={() => {
            resolveDraft(
              'incoming',
              draft.get(),
              path.get(),
              workspace.handle(),
              recovery.handle(),
              errorText.handle()
            );
          }}
        >
          {t('Accept incoming')}
        </Button>
        <Button
          size="small"
          onPress={() => {
            resolveDraft(
              'both',
              draft.get(),
              path.get(),
              workspace.handle(),
              recovery.handle(),
              errorText.handle()
            );
          }}
        >
          {t('Accept both')}
        </Button>
      </View>
      <CodeEditor
        value={draft.get()}
        language={extname(path.get()).slice(1)}
        accessibilityLabel={message('Merge result')}
        className="grow min-h-0"
        onChange={(event) => {
          updateDraft(workspace, recovery, path.get(), event);
        }}
      />
    </View>
    <View hidden={selected.get().mode === 'conflicts'} className="grow min-h-0">
      <Diff patch={output.get().includes('diff --git ') ? output.get().slice(output.get().indexOf('diff --git ')) : ''} mode="split" syntaxHighlight={true} lineNumbers={true} fontSize={font.get()} className="size-full" />
    </View>
  </View>;
  return editor;
}
