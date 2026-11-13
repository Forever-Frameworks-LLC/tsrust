import { searchPageCount } from '../../lib/explorer/search.tsr';
import { FormattingTarget } from '../../lib/workspace/formatting-actions.tsr';
import { Channel, Sender } from 'std:futures';
import { join, extname } from 'std:path';
import {
  Button,
  CodeEditor,
  DiffGutterMarker,
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  Diff,
  Input,
  List,
  ListItem,
  ListItemSuffix,
  Node,
  Tabs,
  Tab,
  Pagination,
  Signal,
  Text,
  Ticker,
  Tooltip,
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
import { hasConflictStart, resolveFirstConflict } from '../../lib/workspace/merge.tsr';

function CommitRow(line: ReadOnly<string>, selected: Signal<GitChange>): Node {
  const label = clone(line);
  const active = computed<boolean>(() => selected.get().mode === 'commit' && selected.get().path === label);
  const style = computed<string>(() => 'px-2 border-b border-border' + (active.get() ? ' bg-primary/20 text-foreground border-l-2 border-primary pl-[6px]' : ''));
  return (
    <ListItem confirmed={active.get()} className={style.get()} value={clone(line)} accessibilityLabel={clone(line)}>
      <Tooltip text={clone(line)} className="grow min-w-0">
        <Text className="truncate">
          {clone(line)}
        </Text>
      </Tooltip>
    </ListItem>
  );
}

function ChangeRow(row: ReadOnly<GitChange>, selected: Signal<GitChange>, font: Signal<int>, pending: Signal<GitRequest>, busy: Signal<boolean>): Node {
  const file = clone(row.path);
  const mode = clone(row.mode);
  const selectedPath = clone(file);
  const active = computed<boolean>(() => selected.get().mode === mode && selected.get().path === selectedPath);
  const rowStyle = computed<string>(() => 'px-2 py-[6px] rounded-none border-b border-border hover:bg-accent' + (active.get() ? ' bg-primary/20 text-foreground border-l-2 border-primary pl-[6px]' : ''));
  const staged = row.mode === 'staged';
  return (
    <ListItem confirmed={active.get()} className={rowStyle.get()} value={row.mode + '/' + row.path} accessibilityLabel={clone(row.path)}>
      <Tooltip text={clone(row.path)} className="grow min-w-0">
        <Text className={'grow min-w-0 truncate text-[' + font.get().toString() + 'px]'}>
          {clone(row.path)}
        </Text>
      </Tooltip>
      <ListItemSuffix>
        <Text className={'shrink-0 text-muted-foreground text-[' + font.get().toString() + 'px]'}>
          {clone(row.status)}
        </Text>
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
  const working = kind === 'working';
  return (
    <View hidden={!changes.get().some((item: ReadOnly<GitChange>) => item.mode === kind)} className="flex-col grow min-h-0">
      <View hidden={working && !changes.get().some((item: ReadOnly<GitChange>) => item.mode !== 'working')}>
        <Text className={'px-2 py-1 text-muted-foreground text-[' + font.get().toString() + 'px]'}>
          {title}
        </Text>
      </View>
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
        {...changes.get().filter((item: ReadOnly<GitChange>) => item.mode === groupKind).map((row: ReadOnly<GitChange>) => ChangeRow(row, selected.handle(), font.handle(), pending.handle(), busy.handle()))}
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
  const query = signal('');
  const page = signal<int>(1);
  const matches = signal<string[]>([]);
  const pages = computed<int>(() => searchPageCount(matches.get().length));
  const checkingBusy = busy.handle();
  const checkingPending = pending.handle();
  const locked = computed<boolean>(() => checkingBusy.get() || checkingPending.get().mode !== '');
  const panelClass = computed<string>(() => 'flex-col size-full min-h-0 text-[' + font.get().toString() + 'px]');
  const sidebar: Node = <View className={panelClass.get()}>
    <Tabs
      variant="underline"
      size="small"
      value={history.get() ? 'history' : 'changes'}
      onChange={(value) => {
        if (locked.get()) return;
        history.set(value === 'history');

        if (history.get()) {
          page.set(1);
          pending.set({
            mode: 'history'
          });
        }
      }}
    >
      <Tab value="changes" label={message('Changes')} />
      <Tab value="history" label={message('History')} />
    </Tabs>
    <View hidden={history.get()} className="shrink-0 h-8 min-w-0">
      <InputGroup size="small" className="w-full h-full min-w-0 rounded-none border-0 border-b border-border">
        <Input
          className={'px-2 text-[' + font.get().toString() + 'px]'}
          value={messageText.get()}
          placeholder={message('Commit message')}
          aria-label={accessibleName('Commit message')}
          onChange={(event) => {
            messageText.set(clone(event.target.value));
          }}
        />
        <InputGroupAddon align="inline-end">
          <InputGroupButton
            icon="check"
            tooltip={message('Commit staged changes')}
            accessibilityLabel={message('Commit staged changes')}
            disabled={locked.get() || messageText.get().trim() === '' || !changes.get().some((row: ReadOnly<GitChange>) => row.mode === 'staged')}
            onClick={() => {
              pending.set({
                mode: 'status', action: 'commit-staged', message: messageText.get()
              });
            }}
          />
          <InputGroupButton
            icon="refresh-cw"
            tooltip={message('Refresh source control')}
            accessibilityLabel={message('Refresh source control')}
            disabled={locked.get()}
            onClick={() => {
              pending.set({
                mode: 'status'
              });
            }}
          />
        </InputGroupAddon>
      </InputGroup>
    </View>
    <View hidden={!history.get()} className="grow flex-col min-h-0">
      <Input
        size="small"
        className="w-full px-2 rounded-none border-0 border-b border-border"
        placeholder={message('Search commits')}
        aria-label={accessibleName('Search commits')}
        value={query.get()}
        onChange={(event) => {
          query.set(clone(event.target.value));
          matches.set(commits.get().filter((line: ReadOnly<string>) => line.toLowerCase().includes(query.get().trim().toLowerCase())));
          page.set(1);
        }}
      />
      <List
        className="grow min-h-0"
        size="small"
        onConfirm={(value) => {
          if (locked.get() || value === '') return;
          selected.set({
            path: clone(value), mode: 'commit', status: ''
          });
          output.set('');
          viewing.set(true);
          pending.set({
            mode: 'commit', commit: value.split(' ')[0]
          });
        }}
      >
        {...matches.get().slice((page.get() - 1) * 50, page.get() * 50).map((line: ReadOnly<string>) => CommitRow(line, selected.handle()))}
      </List>
      <View hidden={pages.get() <= 1}>
        <Pagination
          size="small"
          compact={true}
          page={page.get()}
          total={pages.get() > 0 ? pages.get() : 1}
          onChange={(value) => {
            page.set(value);
          }}
        />
      </View>
    </View>
    <View hidden={errorText.get() === ''}>
      <Text className="px-2 text-destructive">
        {errorText.get()}
      </Text>
    </View>
    <View hidden={history.get()} className="grow flex-col min-h-0">
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
    </View>
    <Ticker interval={150} running={busy.get() || (active.get() === 'git' && visible.get() && pending.get().mode !== '')} onTick={(_frame) => {
      if (busy.get()) {
        match (replies.tryReceive()) {
          Ok(reply) => {
            busy.set(false);
            const req = sent.get();
            if (reply.failed) { errorText.set(localizeError(reply.text)); return; }
            errorText.set('');
            if (req.action === 'commit-staged' && messageText.get() === req.message) messageText.set('');
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
            } else if (reply.mode === 'history') { commits.set(reply.text.split('\n').filter((line: ReadOnly<string>) => line !== '')); matches.set(commits.get().filter((line: ReadOnly<string>) => line.toLowerCase().includes(query.get().trim().toLowerCase()))); page.set(1); }
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

function HunkActions(hunks: ReadOnly<DiffHunk[]>, output: ReadOnly<string>, selection: ReadOnly<GitChange>, pending: Signal<GitRequest>, busy: Signal<boolean>): Node[] {
  const items: Node[] = [];

  if (selection.mode !== 'working' && selection.mode !== 'staged') return items;

  for (const hunk of hunks) {
    const request: GitRequest = {
      mode: clone(selection.mode), action: selection.mode === 'staged' ? 'unstage-hunk' : 'stage-hunk', viewFile: clone(selection.path), hunk: clone(hunk.label), patch: hunk.header + output.slice(hunk.start, hunk.end)
    };
    const staged = selection.mode === 'staged';
    const useOriginal = hunk.modifiedLine === 0;
    items.push(
      <DiffGutterMarker path={clone(selection.path)} side={useOriginal ? 'original' : 'modified'} line={useOriginal ? hunk.originalLine : hunk.modifiedLine}>
        <Button
          icon={staged ? 'minus' : 'plus'}
          variant="ghost"
          size="xsmall"
          className="size-4 p-0"
          tooltip={staged ? translate('Unstage hunk') : translate('Stage hunk')}
          aria-label={staged ? translate('Unstage hunk') : translate('Stage hunk')}
          disabled={busy.get() || pending.get().mode !== ''}
          onPress={() => {
            if (!busy.get() && pending.get().mode === '') pending.set(clone(request));
          }}
        />
      </DiffGutterMarker>
    );
  }

  return items;
}

export function GitEditor(
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>,
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
  const editable = computed<boolean>(() => selected.get().mode === 'conflicts' && workspace.get().files.some((file: ReadOnly<OpenFile>) => file.document.path === path.get() && file.kind === 'text'));
  const checkingBusy = busy.handle();
  const checkingPending = pending.handle();
  const locked = computed<boolean>(() => checkingBusy.get() || checkingPending.get().mode !== '');
  const editor: Node = <View className="size-full flex-col min-w-0 min-h-0 overflow-hidden">
    <View hidden={selected.get().mode !== 'conflicts'} className="flex-col grow min-h-0">
      <View hidden={selected.get().status !== 'DU' && selected.get().status !== 'UD'} className="px-2 shrink-0">
        <Text className={'text-muted-foreground text-[' + font.get().toString() + 'px]'}>
          {selected.get().status === 'DU' ? t('Deleted in current version') : t('Deleted in incoming version')}
        </Text>
      </View>
      <View className="w-full grow min-w-0 min-h-0 overflow-hidden">
        <Diff originalText={ours.get()} modifiedText={theirs.get()} filePath={selected.get().path} mode="split" headerVisible={false} softWrap={true} hunkSeparator="simple" syntaxHighlight={true} fontSize={font.get()} className="size-full" />
      </View>
      <View hidden={!editable.get() || !hasConflictStart(draft.get())} className="flex-row">
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
      <View hidden={!editable.get()} className="flex-col grow min-h-0">
        <CodeEditor
          value={draft.get()}
          language={extname(path.get()).slice(1)}
          accessibilityLabel={message('Merge result')}
          className={'grow min-h-0 font-mono text-[' + font.get().toString() + 'px]'}
          onChange={(event) => {
            updateDraft(workspace, recovery, path.get(), event);
          }}
        />
      </View>
      <View className="shrink-0 flex-row justify-end">
        <Button
          icon="check"
          size="xsmall"
          variant="ghost"
          tooltip={message('Stage resolution')}
          aria-label={accessibleName('Stage resolution')}
          disabled={locked.get()}
          onPress={() => {
            if (selected.get().mode === 'conflicts') {
              const next = workspace.get();
              const index = next.files.findIndex((entry: ReadOnly<OpenFile>) => entry.document.path === path.get() && entry.kind === 'text');

              if (index >= 0) {
                if (hasConflictStart(next.files[index].document.text)) {
                  errorText.set(translate('Resolve conflict markers before staging.'));
                  return;
                }

                const target: FormattingTarget = {
                  path: clone(next.files[index].document.path), session: clone(next.files[index].document.session), revision: next.files[index].document.revision
                };

                match (saveTarget(next, target)) {
                  Err(reason) => {
                    errorText.set(reason);
                    publishWorkspace(next, workspace, recovery);
                    return;
                  },
                  Ok(_) => {}
                }

                publishWorkspace(next, workspace, recovery);
              }
            }

            pending.set({
              mode: 'status', action: selected.get().mode === 'staged' ? 'unstage' : 'stage', path: selected.get().path
            });
          }}
        />
      </View>
    </View>
    <View hidden={selected.get().mode === 'conflicts'} className="w-full grow min-w-0 min-h-0 overflow-hidden">
      <Diff patch={output.get().includes('diff --git ') ? output.get().slice(output.get().indexOf('diff --git ')) : ''} mode="split" headerVisible={false} softWrap={true} hunkSeparator="simple" syntaxHighlight={true} lineNumbers={true} fontSize={font.get()} className="size-full">
        {...HunkActions(hunks.get(), output.get(), selected.get(), pending.handle(), busy.handle())}
      </Diff>
    </View>
  </View>;
  return editor;
}
