import { FormattingTarget } from '../../lib/workspace/formatting-actions.tsr';
import { Channel, Sender } from 'std:futures';
import { join, extname, basename } from 'std:path';
import {
  Button,
  CodeEditor,
  DropdownMenu,
  MenuItem,
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

function ChangeRow(row: ReadOnly<GitChange>, font: Signal<int>, pending: Signal<GitRequest>, busy: Signal<boolean>): Node {
  const file = clone(row.path);
  const staged = row.mode === 'staged';
  return (
    <ListItem value={row.mode + '/' + row.path} accessibilityLabel={clone(row.path)}>
      <Text className={'grow min-w-0 truncate text-[' + font.get().toString() + 'px]'}>
        {basename(row.path)}
      </Text>
      <Text className={'shrink-0 text-muted-foreground text-[' + font.get().toString() + 'px]'}>
        {clone(row.status)}
      </Text>
      <ListItemSuffix>
        <View hidden={row.mode === 'conflicts'}>
          <Button
            icon={staged ? 'minus' : 'plus'}
            size="xsmall"
            variant="ghost"
            disabled={busy.get() || pending.get().mode !== ''}
            tooltip={staged ? translate('Unstage file') : translate('Stage file')}
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
  pending: Signal<GitRequest>,
  busy: Signal<boolean>
): void {
  if (busy.get() || pending.get().mode !== '') return;
  const row = changes.get().find((item: ReadOnly<GitChange>) => item.mode + '/' + item.path === key);

  if (row === undefined) return;
  selected.set(clone(row));
  output.set('');
  hunks.set([]);
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
  font: Signal<int>,
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>,
  viewing: Signal<boolean>,
  changes: Signal<GitChange[]>,
  selected: Signal<GitChange>,
  errorText: Signal<string>,
  output: Signal<string>,
  hunks: Signal<DiffHunk[]>,
  pending: Signal<GitRequest>,
  busy: Signal<boolean>
): Node {
  const groupKind = clone(kind);
  return (
    <View hidden={!changes.get().some((item: ReadOnly<GitChange>) => item.mode === kind)} className="flex-col grow min-h-0">
      <Text className={'px-2 py-1 text-muted-foreground text-[' + font.get().toString() + 'px]'}>
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
            pending.handle(),
            busy.handle()
          );
        }}
      >
        {...changes.get().filter((item: ReadOnly<GitChange>) => item.mode === groupKind).map((row: ReadOnly<GitChange>) => ChangeRow(row, font.handle(), pending.handle(), busy.handle()))}
      </List>
    </View>
  );
}

export function GitPanel(
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>,
  font: Signal<int>,
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
  const sidebar: Node = <View className="size-full flex-col min-h-0 min-w-0 overflow-hidden">
    <View className="flex-row items-center min-w-0 shrink-0 px-1 gap-1">
      <Input
        className={'grow min-w-0 text-[' + font.get().toString() + 'px]'}
        value={messageText.get()}
        placeholder={message('Commit message')}
        aria-label={accessibleName('Commit message')}
        onChange={(event) => {
          messageText.set(clone(event.target.value));
        }}
      />
      <Button
        icon="check"
        size="xsmall"
        variant="ghost"
        tooltip={message('Commit staged changes')}
        aria-label={accessibleName('Commit staged changes')}
        disabled={locked.get() || messageText.get().trim() === '' || !changes.get().some((row: ReadOnly<GitChange>) => row.mode === 'staged')}
        onPress={() => {
          pending.set({
            mode: 'status', action: 'commit-staged', message: messageText.get()
          });
        }}
      />
      <Button
        icon="history"
        variant="ghost"
        size="xsmall"
        tooltip={message('History')}
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
        tooltip={message('Refresh source control')}
        aria-label={accessibleName('Refresh source control')}
        disabled={locked.get()}
        onPress={() => {
          pending.set({
            mode: 'status'
          });
        }}
      />
    </View>
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
      font.handle(),
      workspace.handle(),
      recovery.handle(),
      viewing.handle(),
      changes.handle(),
      selected.handle(),
      errorText.handle(),
      output.handle(),
      hunks.handle(),
      pending.handle(),
      busy.handle()
    )}
    {ChangeGroup(
      'staged',
      translate('Staged changes'),
      font.handle(),
      workspace.handle(),
      recovery.handle(),
      viewing.handle(),
      changes.handle(),
      selected.handle(),
      errorText.handle(),
      output.handle(),
      hunks.handle(),
      pending.handle(),
      busy.handle()
    )}
    {ChangeGroup(
      'working',
      translate('Changes'),
      font.handle(),
      workspace.handle(),
      recovery.handle(),
      viewing.handle(),
      changes.handle(),
      selected.handle(),
      errorText.handle(),
      output.handle(),
      hunks.handle(),
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
            else {ours.set(reply.ours ?? '');theirs.set(reply.theirs ?? '');hunks.set(diffHunks(reply.text));output.set(clone(reply.text));if (req.action !== undefined) pending.set({mode:'status'});}
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

function HunkAction(item: ReadOnly<DiffHunk>, selected: Signal<GitChange>, output: Signal<string>, pending: Signal<GitRequest>, busy: Signal<boolean>): Node {
  const label = clone(item.label);
  return (
    <MenuItem
      label={label.slice(label.indexOf('  @@ ') + 2)}
      onSelect={() => {
        if (busy.get() || pending.get().mode !== '') return;
        const patch = output.get();
        const current = diffHunks(patch).find((entry: ReadOnly<DiffHunk>) => entry.label === label);

        if (current !== undefined) pending.set({
          mode: selected.get().mode, action: selected.get().mode === 'staged' ? 'unstage-hunk' : 'stage-hunk', viewFile: selected.get().path, hunk: clone(label), patch: current.header + patch.slice(current.start, current.end)
        });
      }}
    />
  );
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
  hunks: Signal<DiffHunk[]>
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
  const editor: Node = <View className="size-full flex-col min-w-0 min-h-0 overflow-hidden">
    <View className="w-full shrink-0 flex-row items-center min-w-0 px-2 border-b">
      <Text className={'grow min-w-0 truncate font-mono text-[' + font.get().toString() + 'px]'}>
        {basename(selected.get().path)}
      </Text>
      <View hidden={selected.get().mode !== 'conflicts'}>
        <Button icon="check" size="xsmall" variant="ghost" tooltip={message('Stage resolution')} aria-label={accessibleName('Stage resolution')} disabled={locked.get()} onPress={() => {
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
      }} />
      </View>
      <View hidden={hunks.get().length === 0 || !['working', 'staged'].includes(selected.get().mode)}>
        <DropdownMenu fontSize={font.get()}>
          <Button icon="ellipsis" variant="ghost" size="xsmall" disabled={locked.get()} tooltip={selected.get().mode === 'staged' ? translate('Unstage hunk') : translate('Stage hunk')} aria-label={selected.get().mode === 'staged' ? translate('Unstage hunk') : translate('Stage hunk')} />
          {...hunks.get().map((item: ReadOnly<DiffHunk>) => HunkAction(item, selected.handle(), output.handle(), pending.handle(), busy.handle()))}
        </DropdownMenu>
      </View>
      <Button
        icon="x"
        size="xsmall"
        variant="ghost"
        tooltip={message('Close diff')}
        aria-label={accessibleName('Close diff')}
        onPress={() => {
          viewing.set(false);
        }}
      />
    </View>
    <View hidden={selected.get().mode !== 'conflicts'} className="flex-col grow min-h-0">
      <View className="w-full grow min-w-0 min-h-0 overflow-hidden">
        <Diff originalText={ours.get()} modifiedText={theirs.get()} filePath={selected.get().path} mode="split" headerVisible={false} softWrap={true} hunkSeparator="simple" syntaxHighlight={true} fontSize={font.get()} className="size-full" />
      </View>
      <View className="flex-row">
        <Button
          variant="ghost"
          size="small"
          tooltip={message('Accept current')}
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
          variant="ghost"
          size="small"
          tooltip={message('Accept incoming')}
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
          variant="ghost"
          size="small"
          tooltip={message('Accept both')}
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
    <View hidden={selected.get().mode === 'conflicts'} className="w-full grow min-w-0 min-h-0 overflow-hidden">
      <Diff patch={output.get().includes('diff --git ') ? output.get().slice(output.get().indexOf('diff --git ')) : ''} mode="split" headerVisible={false} softWrap={true} hunkSeparator="simple" syntaxHighlight={true} lineNumbers={true} fontSize={font.get()} className="size-full" />
    </View>
  </View>;
  return editor;
}
