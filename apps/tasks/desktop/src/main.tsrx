import { fromUtf8, toHex } from 'std:bytes';
import { randomBytes, sha256 } from 'std:crypto';
import { get, homeDir } from 'std:env';
import { mkdirAll } from 'std:fs';
import { Channel, Sender } from 'std:futures';
import { join } from 'std:path';
import {
  View,
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetFooter,
  TitleBar,
  Tag,
  DataTableCell,
  Text,
  Button,
  Input,
  InputGroup,
  InputGroupAddon,
  InputChangeEvent,
  Textarea,
  Select,
  SelectItem,
  DataTable,
  DataTableColumn,
  DataTableEmpty,
  List,
  ListItem,
  ListEmpty,
  Pagination,
  Ticker,
  Shortcut,
  Node,
  Signal,
  signal,
  run,
  setTheme
} from 'std:ui';

import { TaskRecord, WriteTask } from '../../shared/tasks.tsr';
import { LocalDraft, DraftJournal, recoverDraft, persistDraft } from './drafts.tsr';
import {
  TaskQuery,
  TaskPage,
  initialQuery,
  headerSort,
  taskPage,
  emptyDraft,
  taskDraft,
  tableRows,
  statusLabel,
  priorityLabel
} from './model.tsr';
import { TaskCommand, TaskUpdate, taskService } from './service.tsr';

interface Editor {
  open: boolean;
  id: string;
  value: WriteTask;
  dirty: boolean;
  confirm: string;
  conflict: boolean;
  current: TaskRecord | null;
  message: string;
}

interface Workspace {
  tasks: Signal<TaskRecord[]>;
  query: Signal<TaskQuery>;
  page: Signal<TaskPage>;
  editor: Signal<Editor>;
  online: Signal<boolean>;
  ready: Signal<boolean>;
  status: Signal<string>;
  busy: Signal<boolean>;
  requestId: Signal<int>;
  selected: Signal<string>;
  mode: Signal<string>;
  font: Signal<int>;
  journal: Signal<DraftJournal>;
  cursor: Signal<int>;
  commands: Sender<TaskCommand>;
}

function refreshPage(ui: ReadOnly<Workspace>): void {
  const page = taskPage(ui.tasks.get(), ui.query.get());

  if (!page.items.some((task) => task.id === ui.selected.get())) ui.selected.set('');
  ui.page.set(page);
}

function saveLocalDraft(ui: ReadOnly<Workspace>): boolean {
  const editor = ui.editor.get();
  const local: LocalDraft = {
    id: clone(editor.id), value: clone(editor.value)
  };
  const journal = ui.journal.get();

  match (persistDraft(journal, editor.open && editor.dirty ? local : null)) {
    Ok(_) => {
      ui.journal.set(journal);
      return true;
    },
    Err(message) => {
      ui.journal.set(journal);
      ui.status.set('Draft backup failed. Keep this window open. ' + message);
      return false;
    }
  }
}

function openTask(ui: ReadOnly<Workspace>, id: ReadOnly<string>): void {
  const task = ui.tasks.get().find((task) => task.id === id);

  if (task === undefined) return;
  ui.editor.set({
    open: true, id: clone(task.id), value: taskDraft(task), dirty: false, confirm: '', conflict: false, current: null, message: ''
  });
}

function editDraft(ui: ReadOnly<Workspace>, value: WriteTask): void {
  const editor = ui.editor.get();
  editor.value = value;
  editor.dirty = true;
  editor.confirm = '';
  ui.editor.set(editor);
  saveLocalDraft(ui);
}

function closeEditor(ui: ReadOnly<Workspace>, discard: boolean = false): void {
  const editor = ui.editor.get();

  if (editor.dirty && !discard) {
    editor.confirm = 'discard';
    ui.editor.set(editor);
    return;
  }

  editor.open = false;
  editor.dirty = false;
  editor.confirm = '';
  ui.editor.set(editor);
  saveLocalDraft(ui);
}

function submit(ui: ReadOnly<Workspace>, remove: boolean = false): void {
  if (ui.busy.get() || !ui.online.get()) return;
  const editor = ui.editor.get();
  const requestId = ui.requestId.get() + 1;
  let command: TaskCommand = {
    kind: 'save', requestId, id: clone(editor.id), value: clone(editor.value)
  };

  if (remove) command = {
    kind: 'delete', requestId, id: clone(editor.id), expectedVersion: editor.value.expectedVersion
  };

  match (ui.commands.trySend(command)) {
    Ok(_) => {
      ui.requestId.set(requestId);
      ui.busy.set(true);
      editor.message = remove ? 'Deleting…' : 'Saving…';
      editor.confirm = '';
      ui.editor.set(editor);
    },
    Err(_) => {
      editor.message = 'The service is busy. Your draft is safe; try again.';
      ui.editor.set(editor);
    }
  }
}

function acceptUpdate(ui: ReadOnly<Workspace>, update: TaskUpdate): void {
  if (update.ready && update.snapshot.cursor !== ui.cursor.get()) {
    ui.cursor.set(update.snapshot.cursor);
    ui.tasks.set(update.snapshot.tasks);
    refreshPage(ui);
  }

  ui.ready.set(update.ready);
  ui.online.set(update.connected);
  ui.status.set(clone(update.message));

  if (update.requestId === 0 || update.requestId !== ui.requestId.get()) return;
  ui.busy.set(false);
  const editor = ui.editor.get();
  editor.message = update.message;

  if (update.outcome === 'saved' || update.outcome === 'deleted') {
    editor.open = false;
    editor.dirty = false;
    editor.conflict = false;
    ui.editor.set(editor);
    saveLocalDraft(ui);
  } else {
    editor.conflict = update.outcome === 'conflict';
    editor.current = update.current;
    ui.editor.set(editor);
  }
}

function conflictText(editor: ReadOnly<Editor>): string {
  const current = clone(editor.current);

  if (current === null) return 'The server no longer has this task. Your draft is preserved.';
  return 'Server version ' + current.version.toString() + ': ' + current.title + '\n' + statusLabel(current.status) + ' · ' + priorityLabel(current.priority) + ' · ' + (current.assignee === '' ? 'Unassigned' : current.assignee) + ' · ' + (current.due_date ?? 'No due date') + '\n' + current.description;
}

function workspaceHandles(ui: ReadOnly<Workspace>): Workspace {
  return {
    tasks: ui.tasks.handle(), query: ui.query.handle(), page: ui.page.handle(), editor: ui.editor.handle(), online: ui.online.handle(), ready: ui.ready.handle(), status: ui.status.handle(), busy: ui.busy.handle(), requestId: ui.requestId.handle(), selected: ui.selected.handle(), mode: ui.mode.handle(), font: ui.font.handle(), journal: ui.journal.handle(), cursor: ui.cursor.handle(), commands: ui.commands.share()
  };
}

// Retained actions own weak state handles, so callbacks cannot keep the component alive.
function action(ui: Owned<Workspace>, perform: Owned<Function<void, [ui: ReadOnly<Workspace>]>>): Function<void> {
  return () => {
    perform(ui);
  };
}

function eventAction<T>(ui: Owned<Workspace>, perform: Owned<Function<void, [ui: ReadOnly<Workspace>, value: ReadOnly<T>]>>): Function<void, [value: ReadOnly<T>]> {
  return (value: ReadOnly<T>) => {
    perform(ui, value);
  };
}

function StatusTag(task: ReadOnly<TaskRecord>, appearance: Signal<string>, font: Signal<int>): Node {
  const fillStatus = clone(task.status);
  const textStatus = clone(task.status);
  const label = statusLabel(task.status);
  return (
    <Tag size="small" className={'text-[' + font.get().toString() + 'px]'} color={appearance.get() === 'dark' ? fillStatus === 'done' ? '#163827' : fillStatus === 'in_progress' ? '#172d4e' : '#303641' : fillStatus === 'done' ? '#dcfce7' : fillStatus === 'in_progress' ? '#dbeafe' : '#e2e8f0'} foreground={appearance.get() === 'dark' ? textStatus === 'done' ? '#bbf7d0' : textStatus === 'in_progress' ? '#bfdbfe' : '#f1f5f9' : textStatus === 'done' ? '#166534' : textStatus === 'in_progress' ? '#1e40af' : '#334155'}>
      {label}
    </Tag>
  );
}

function PriorityTag(task: ReadOnly<TaskRecord>, appearance: Signal<string>, font: Signal<int>): Node {
  const priority = task.priority;
  const label = priorityLabel(priority);
  return (
    <Tag size="small" className={'text-[' + font.get().toString() + 'px]'} color={appearance.get() === 'dark' ? priority === 0 ? '#451a1a' : priority === 1 ? '#422a0d' : '#303641' : priority === 0 ? '#fee2e2' : priority === 1 ? '#fef3c7' : '#e2e8f0'} foreground={appearance.get() === 'dark' ? priority === 0 ? '#fecaca' : priority === 1 ? '#fde68a' : '#f1f5f9' : priority === 0 ? '#991b1b' : priority === 1 ? '#92400e' : '#334155'}>
      {label}
    </Tag>
  );
}

function App(commands: Sender<TaskCommand>, updates: Channel<TaskUpdate>, journalPath: string): Node {
  const recovered = recoverDraft(journalPath);
  const tasksState = signal<TaskRecord[]>([]);
  const queryState = signal(initialQuery());
  const pageState = signal<TaskPage>({
    items: [], total: 0, pages: 1, page: 1
  });
  const editorState = signal<Editor>({
    open: recovered.draft !== null, id: recovered.draft === null ? '' : clone(recovered.draft.id), value: recovered.draft === null ? emptyDraft() : clone(recovered.draft.value), dirty: recovered.draft !== null, confirm: '', conflict: false, current: null, message: recovered.draft === null ? '' : 'Recovered your local draft.'
  });
  const onlineState = signal(false);
  const readyState = signal(false);
  const statusState = signal(recovered.error === '' ? 'Connecting…' : clone(recovered.error));
  const busyState = signal(false);
  const requestIdState = signal(0);
  const selectedState = signal('');
  const modeState = signal('table');
  const themeState = signal('dark');
  const fontState = signal(14);
  const journalState = signal(recovered.journal);
  const cursorState = signal(-1);
  const shared: Workspace = {
    tasks: tasksState.handle(), query: queryState.handle(), page: pageState.handle(), editor: editorState.handle(), online: onlineState.handle(), ready: readyState.handle(), status: statusState.handle(), busy: busyState.handle(), requestId: requestIdState.handle(), selected: selectedState.handle(), mode: modeState.handle(), font: fontState.handle(), journal: journalState.handle(), cursor: cursorState.handle(), commands
  };
  return (
    <View className={'size-full flex-col bg-background text-foreground text-[' + fontState.get().toString() + 'px]'}>
      <Ticker
        interval={100}
        onTick={eventAction<int>(workspaceHandles(shared), (ui, _) => {
          while (true) {
            match (updates.tryReceive()) {
              Ok(update) => {
                acceptUpdate(ui, update);
              },
              Err(error) => {
                if (error.code !== 'empty') {
                  onlineState.set(false);
                  statusState.set('The background service stopped. Restart Worklist to reconnect.');
                }

                return;
              }
            }
          }
        })}
      />
      <Shortcut
        command="zoom-in"
        keys="secondary-="
        onTrigger={() => {
          const next = fontState.get() + 1;
          fontState.set(next > 24 ? 24 : next);
        }}
      />
      <Shortcut
        command="zoom-out"
        keys="secondary--"
        onTrigger={() => {
          const next = fontState.get() - 1;
          fontState.set(next < 11 ? 11 : next);
        }}
      />
      <Shortcut
        command="reset-zoom"
        keys="secondary-0"
        onTrigger={() => {
          fontState.set(14);
        }}
      />
      <Shortcut
        command="save-task"
        keys="secondary-s"
        onTrigger={action(workspaceHandles(shared), (ui) => {
          if (editorState.get().open) submit(ui);
        })}
      />
      <TitleBar>
        <View className="h-full grow min-w-0 pr-3 flex-row items-center gap-3">
          <Text className={onlineState.get() ? 'text-green-700 dark:text-green-400 truncate' : 'text-amber-700 dark:text-amber-300 truncate'}>
            {statusState.get()}
          </Text>
          <View className="grow" />
          <View hidden={editorState.get().open}>
            <Button className={'shrink-0 text-[' + fontState.get().toString() + 'px]'} size="small" icon="plus" tooltip="Create task" disabled={!journalState.get().ready} onPress={() => {
            editorState.set({open: true, id: unwrap(toHex(unwrap(randomBytes(16)))), value: emptyDraft(), dirty: false, confirm: '', conflict: false, current: null, message: ''});
          }}>New task</Button>
          </View>
          <Button
            className={'shrink-0 text-[' + fontState.get().toString() + 'px]'}
            icon={themeState.get() === 'light' ? 'moon' : 'sun'}
            size="small"
            variant="ghost"
            tooltip="Toggle light and dark theme"
            aria-label="Toggle light and dark theme"
            onPress={() => {
              if (themeState.get() === 'light') {
                setTheme('dark');
                themeState.set('dark');
              } else {
                setTheme('light');
                themeState.set('light');
              }
            }}
          />
          <Button
            className={'shrink-0 text-[' + fontState.get().toString() + 'px]'}
            icon="refresh-cw"
            size="small"
            variant="ghost"
            tooltip="Sync tasks"
            aria-label="Sync tasks"
            onPress={action(workspaceHandles(shared), (ui) => {
              match (ui.commands.trySend({
                kind: 'refresh'
              })) {
                Ok(_) => {},
                Err(_) => {
                  statusState.set('Sync is already queued.');
                }
              }
            })}
          />
        </View>
      </TitleBar>
      <View hidden={journalState.get().ready} role="alert" className="px-6 py-3 bg-muted border-b border-border flex-col gap-1">
        <Text className="text-destructive">Draft backup is unavailable. The existing file has been preserved.</Text>
        <Text className="text-muted-foreground">
          {journalState.get().path}
        </Text>
      </View>
      <View className="flex-col grow min-h-0 min-w-0">
        <InputGroup focusRing={false} size="small" accessibilityLabel="Task search and filters" className={'w-full h-[' + (fontState.get() > 18 ? (fontState.get() as number) * 1.5 + 20 : 48).toString() + 'px] shrink-0 border-0 border-b border-border rounded-none bg-[' + (themeState.get() === 'light' ? '#fafafa' : '#17171766') + '] text-[' + fontState.get().toString() + 'px]'}>
          <Input
            size="small"
            appearance={false}
            bordered={false}
            aria-label="Search tasks"
            placeholder="Search tasks, notes or people"
            prefix="search"
            value={queryState.get().search}
            className={'grow min-w-0 ' + 'text-[' + fontState.get().toString() + 'px]'}
            onChange={eventAction<InputChangeEvent>(workspaceHandles(shared), (ui, event) => {
              const query = queryState.get();
              query.search = clone(event.target.value);
              query.page = 1;
              queryState.set(query);
              refreshPage(ui);
            })}
          />
          <InputGroupAddon align="inline-end" className="px-3 py-2 gap-3">
            <Select
              appearance={false}
              accessibilityLabel="Filter by status"
              value={queryState.get().status}
              size="small"
              className={'w-[' + (fontState.get() * 10 + 40).toString() + 'px] shrink-0 text-[' + fontState.get().toString() + 'px]'}
              onChange={eventAction<string>(workspaceHandles(shared), (ui, value) => {
                const query = queryState.get();
                query.status = clone(value);
                query.page = 1;
                queryState.set(query);
                refreshPage(ui);
              })}
            >
              <SelectItem value="all">All statuses</SelectItem>
              <SelectItem value="todo">To do</SelectItem>
              <SelectItem value="in_progress">In progress</SelectItem>
              <SelectItem value="done">Done</SelectItem>
            </Select>
            <View hidden={modeState.get() !== 'cards'} className="flex-row items-center gap-2 shrink-0">
              <Select
                appearance={false}
                accessibilityLabel="Sort cards"
                size="small"
                value={queryState.get().sort}
                className={'w-[' + (fontState.get() * 10 + 40).toString() + 'px] shrink-0 text-[' + fontState.get().toString() + 'px]'}
                onChange={eventAction<string>(workspaceHandles(shared), (ui, value) => {
                  const query = queryState.get();
                  query.sort = clone(value);

                  if (value === 'original') query.descending = false;
                  query.page = 1;
                  queryState.set(query);
                  refreshPage(ui);
                })}
              >
                <SelectItem value="original">Original order</SelectItem>
                <SelectItem value="updated">Last updated</SelectItem>
                <SelectItem value="title">Title</SelectItem>
                <SelectItem value="status">Status</SelectItem>
                <SelectItem value="priority">Priority</SelectItem>
                <SelectItem value="assignee">Assignee</SelectItem>
                <SelectItem value="due">Due date</SelectItem>
              </Select>
              <Button
                className={'shrink-0 text-[' + fontState.get().toString() + 'px]'}
                size="small"
                variant="ghost"
                disabled={queryState.get().sort === 'original'}
                icon={queryState.get().descending ? 'arrow-down' : 'arrow-up'}
                tooltip={queryState.get().descending ? 'Sort ascending' : 'Sort descending'}
                aria-label={queryState.get().descending ? 'Sort ascending' : 'Sort descending'}
                onPress={action(workspaceHandles(shared), (ui) => {
                  const query = queryState.get();
                  query.descending = !query.descending;
                  query.page = 1;
                  queryState.set(query);
                  refreshPage(ui);
                })}
              />
            </View>
            <Button
              className={'shrink-0 text-[' + fontState.get().toString() + 'px]'}
              variant="ghost"
              icon="table"
              size="small"
              selected={modeState.get() === 'table'}
              toggled={modeState.get() === 'table'}
              tooltip="Table view"
              aria-label="Table view"
              onPress={() => {
                modeState.set('table');
              }}
            />
            <Button
              className={'shrink-0 text-[' + fontState.get().toString() + 'px]'}
              variant="ghost"
              icon="layout-grid"
              size="small"
              selected={modeState.get() === 'cards'}
              toggled={modeState.get() === 'cards'}
              tooltip="Card view"
              aria-label="Card view"
              onPress={() => {
                modeState.set('cards');
              }}
            />
          </InputGroupAddon>
        </InputGroup>
        <View hidden={modeState.get() !== 'table'} className="grow min-h-0 min-w-0">
          <DataTable
            className={'size-full ' + 'text-[' + fontState.get().toString() + 'px]'}
            rows={tableRows(pageState.get())}
            size="small"
            bordered={false}
            sortable={true}
            manualSorting={true}
            scrollResetKey={JSON.stringify(queryState.get())}
            columnMovable={false}
            fixedColumnsBorder={true}
            accessibilityLabel="Worklist tasks"
            loading={!readyState.get()}
            onSelect={eventAction<int>(workspaceHandles(shared), (ui, row) => {
              const items = pageState.get().items;

              if (row >= 0 && row < items.length) openTask(ui, items[row].id);
            })}
            onSort={eventAction<string[]>(workspaceHandles(shared), (ui, sort) => {
              const query = queryState.get();
              headerSort(query, sort[0], sort[1]);
              queryState.set(query);
              refreshPage(ui);
            })}
            onActivate={eventAction<int>(workspaceHandles(shared), (ui, row) => {
              const items = pageState.get().items;

              if (row >= 0 && row < items.length) openTask(ui, items[row].id);
            })}
          >
            <DataTableColumn sort={queryState.get().sort === 'title' ? queryState.get().descending ? 'descending' : 'ascending' : 'none'} sortable={true} key="0" fill={true} fixed="left" width={340} minWidth={340}>Task</DataTableColumn>
            <DataTableColumn sort={queryState.get().sort === 'status' ? queryState.get().descending ? 'descending' : 'ascending' : 'none'} sortable={true} key="1" width={145}>Status</DataTableColumn>
            <DataTableColumn sort={queryState.get().sort === 'priority' ? queryState.get().descending ? 'descending' : 'ascending' : 'none'} sortable={true} key="2" width={130}>Priority</DataTableColumn>
            <DataTableColumn sort={queryState.get().sort === 'assignee' ? queryState.get().descending ? 'descending' : 'ascending' : 'none'} sortable={true} key="3" width={160}>Assignee</DataTableColumn>
            <DataTableColumn sort={queryState.get().sort === 'due' ? queryState.get().descending ? 'descending' : 'ascending' : 'none'} sortable={true} key="4" width={140}>Due date</DataTableColumn>
            <DataTableColumn sort={queryState.get().sort === 'updated' ? queryState.get().descending ? 'descending' : 'ascending' : 'none'} sortable={true} key="5" width={190}>Updated</DataTableColumn>
            {...pageState.get().items.map((task: ReadOnly<TaskRecord>, row: int) => <DataTableCell row={row} column="1">
              {StatusTag(task, themeState.handle(), fontState.handle())}
            </DataTableCell>
            )}
            {...pageState.get().items.map((task: ReadOnly<TaskRecord>, row: int) => <DataTableCell row={row} column="2">
              {PriorityTag(task, themeState.handle(), fontState.handle())}
            </DataTableCell>
            )}
            <DataTableEmpty>
              <View className="flex-col p-8 gap-2">
                <Text className="font-semibold">
                  {readyState.get() ? 'No tasks in this view' : 'Loading your workspace…'}
                </Text>
                <Text className="text-muted-foreground">
                  {readyState.get() ? 'Clear your search or create the next thing to work on.' : 'Saved tasks will remain available when the server is offline.'}
                </Text>
              </View>
            </DataTableEmpty>
          </DataTable>
        </View>
        <View hidden={modeState.get() !== 'cards'} className="grow min-h-0 px-3 w-full">
          <List
            className={'size-full ' + 'text-[' + fontState.get().toString() + 'px]'}
            selectable={false}
            minItemWidth={340}
            scrollResetKey={JSON.stringify(queryState.get())}
            size="small"
            onConfirm={eventAction<string>(workspaceHandles(shared), (ui, id) => {
              openTask(ui, id);
            })}
          >
            {...pageState.get().items.map((task: ReadOnly<TaskRecord>) => <ListItem value={clone(task.id)} accessibilityLabel={clone(task.title)} className={'w-full min-w-0 mb-3 p-4 h-[' + (fontState.get() * 8 + 48).toString() + 'px] text-[' + fontState.get().toString() + 'px] border border-border rounded-md bg-card'}>
              <View className="flex-col grow min-w-0 gap-2">
                <View className="flex-row items-center gap-3">
                  <Text className="font-medium grow min-w-0 line-clamp-2">
                    {clone(task.title)}
                  </Text>
                  {PriorityTag(task, themeState.handle(), fontState.handle())}
                </View>
                <Text className="text-muted-foreground truncate">{task.description === '' ? 'No notes yet.' : task.description.replaceAll('\n', ' ')}</Text>
                <View className="flex-row items-center gap-2">
                  {StatusTag(task, themeState.handle(), fontState.handle())}
                  <Text className="text-muted-foreground">
                    {task.assignee === '' ? 'Unassigned' : clone(task.assignee)}
                  </Text>
                  <View className="grow" />
                  <Text className="text-muted-foreground">
                    {clone(task.due_date ?? 'No due date')}
                  </Text>
                </View>
              </View>
            </ListItem>
            )}
            <ListEmpty>
              <View className="p-8 flex-col gap-2">
                <Text className="font-semibold">No tasks in this view</Text>
                <Text className="text-muted-foreground">Clear the filters or create something new.</Text>
              </View>
            </ListEmpty>
          </List>
        </View>
        <View className={'h-[' + (fontState.get() > 18 ? (fontState.get() as number) * 1.5 + 20 : 48).toString() + 'px] shrink-0 px-3 flex-row items-center gap-3'}>
          <Text className="text-muted-foreground">
            {pageState.get().total.toString() + ' tasks'}
          </Text>
          <View className="grow" />
          <View hidden={pageState.get().pages <= 1}>
            <Pagination
              className={'shrink-0 text-[' + fontState.get().toString() + 'px]'}
              page={pageState.get().page}
              total={pageState.get().pages}
              visiblePages={7}
              size="small"
              onChange={eventAction<int>(workspaceHandles(shared), (ui, page) => {
                const query = queryState.get();
                query.page = page;
                queryState.set(query);
                refreshPage(ui);
              })}
            />
          </View>
          <View className="grow" />
          <Select
            accessibilityLabel="Tasks per page"
            value={queryState.get().pageSize.toString()}
            className={'w-[' + (fontState.get() * 8 + 48).toString() + 'px] shrink-0 text-[' + fontState.get().toString() + 'px]'}
            size="small"
            onChange={eventAction<string>(workspaceHandles(shared), (ui, value) => {
              const query = queryState.get();
              query.pageSize = value === '0' ? 0 : value === '25' ? 25 : value === '500' ? 500 : 100;
              query.page = 1;
              queryState.set(query);
              refreshPage(ui);
            })}
          >
            <SelectItem value="25">25 per page</SelectItem>
            <SelectItem value="100">100 per page</SelectItem>
            <SelectItem value="500">500 per page</SelectItem>
            <SelectItem value="0">All tasks</SelectItem>
          </Select>
        </View>
      </View>
      <Sheet
        open={editorState.get().open}
        side="right"
        size={560}
        resizable={true}
        overlayClosable={false}
        title={editorState.get().value.expectedVersion === 0 ? 'New task' : 'Edit task'}
        onOpenChange={eventAction<boolean>(workspaceHandles(shared), (ui, open) => {
          if (!open && !busyState.get()) closeEditor(ui);
        })}
      >
        <SheetContent>
          <SheetHeader>
            <SheetTitle>
              <Text className={'font-medium text-[' + fontState.get().toString() + 'px]'}>
                {editorState.get().value.expectedVersion === 0 ? 'New task' : 'Edit task'}
              </Text>
            </SheetTitle>
          </SheetHeader>
          <View className={'flex-col gap-4 py-4 w-full text-[' + fontState.get().toString() + 'px]'}>
            <View className="flex-col gap-2">
              <Text className="font-medium">Title</Text>
              <Input
                className={'shrink-0 text-[' + fontState.get().toString() + 'px]'}
                size="small"
                aria-label="Task title"
                placeholder="What needs to happen?"
                disabled={busyState.get()}
                value={editorState.get().value.title}
                onChange={eventAction<InputChangeEvent>(workspaceHandles(shared), (ui, event) => {
                  const draft = editorState.get().value;
                  draft.title = clone(event.target.value);
                  editDraft(ui, draft);
                })}
              />
            </View>
            <View className="flex-row gap-4">
              <View className="flex-col gap-2 grow">
                <Text className="font-medium">Status</Text>
                <Select
                  className={'shrink-0 text-[' + fontState.get().toString() + 'px]'}
                  size="small"
                  accessibilityLabel="Task status"
                  disabled={busyState.get()}
                  value={editorState.get().value.status}
                  onChange={eventAction<string>(workspaceHandles(shared), (ui, value) => {
                    const draft = editorState.get().value;
                    draft.status = value === 'done' ? 'done' : value === 'in_progress' ? 'in_progress' : 'todo';
                    editDraft(ui, draft);
                  })}
                >
                  <SelectItem value="todo">To do</SelectItem>
                  <SelectItem value="in_progress">In progress</SelectItem>
                  <SelectItem value="done">Done</SelectItem>
                </Select>
              </View>
              <View className="flex-col gap-2 grow">
                <Text className="font-medium">Priority</Text>
                <Select
                  className={'shrink-0 text-[' + fontState.get().toString() + 'px]'}
                  size="small"
                  accessibilityLabel="Task priority"
                  disabled={busyState.get()}
                  value={editorState.get().value.priority.toString()}
                  onChange={eventAction<string>(workspaceHandles(shared), (ui, value) => {
                    const draft = editorState.get().value;
                    draft.priority = value === '0' ? 0 : value === '1' ? 1 : value === '3' ? 3 : 2;
                    editDraft(ui, draft);
                  })}
                >
                  <SelectItem value="0">Urgent</SelectItem>
                  <SelectItem value="1">High</SelectItem>
                  <SelectItem value="2">Normal</SelectItem>
                  <SelectItem value="3">Low</SelectItem>
                </Select>
              </View>
            </View>
            <View className="flex-row gap-4">
              <View className="flex-col gap-2 grow">
                <Text className="font-medium">Assignee</Text>
                <Input
                  className={'shrink-0 text-[' + fontState.get().toString() + 'px]'}
                  size="small"
                  aria-label="Task assignee"
                  placeholder="Unassigned"
                  value={editorState.get().value.assignee}
                  disabled={busyState.get()}
                  onChange={eventAction<InputChangeEvent>(workspaceHandles(shared), (ui, event) => {
                    const draft = editorState.get().value;
                    draft.assignee = clone(event.target.value);
                    editDraft(ui, draft);
                  })}
                />
              </View>
              <View className="flex-col gap-2 grow">
                <Text className="font-medium">Due date</Text>
                <Input
                  className={'shrink-0 text-[' + fontState.get().toString() + 'px]'}
                  size="small"
                  aria-label="Task due date"
                  placeholder="YYYY-MM-DD"
                  value={editorState.get().value.due_date ?? ''}
                  disabled={busyState.get()}
                  onChange={eventAction<InputChangeEvent>(workspaceHandles(shared), (ui, event) => {
                    const draft = editorState.get().value;
                    draft.due_date = event.target.value === '' ? null : clone(event.target.value);
                    editDraft(ui, draft);
                  })}
                />
              </View>
            </View>
            <View className="flex-col gap-2">
              <Text className="font-medium">Notes</Text>
              <Textarea
                className={'shrink-0 text-[' + fontState.get().toString() + 'px]'}
                size="small"
                aria-label="Task notes"
                rows={8}
                value={editorState.get().value.description}
                disabled={busyState.get()}
                placeholder="Context, links, and a clear finish line."
                onChange={eventAction<InputChangeEvent>(workspaceHandles(shared), (ui, event) => {
                  const draft = editorState.get().value;
                  draft.description = clone(event.target.value);
                  editDraft(ui, draft);
                })}
              />
            </View>
            <View hidden={editorState.get().message === ''} className="p-4 rounded-md bg-muted flex-col gap-3" role="status">
              <Text>
                {editorState.get().message}
              </Text>
              <View hidden={!editorState.get().conflict} className="flex-col gap-3">
                <Text className="text-muted-foreground">
                  {conflictText(editorState.get())}
                </Text>
                <View className="flex-row gap-2">
                  <Button className={'shrink-0 text-[' + fontState.get().toString() + 'px]'} size="small" disabled={editorState.get().current === null} onPress={action(workspaceHandles(shared), (ui) => { const current = editorState.get().current; if (current !== null) { const editor = editorState.get(); editor.value = taskDraft(current); editor.conflict = false; editor.dirty = false; editor.message = ''; editorState.set(editor); saveLocalDraft(ui); } })}>Use server version</Button>
                  <Button className={'shrink-0 text-[' + fontState.get().toString() + 'px]'} size="small" disabled={!onlineState.get() || busyState.get()} onPress={action(workspaceHandles(shared), (ui) => { const editor = editorState.get(); editor.value.expectedVersion = editor.current === null ? 0 : editor.current.version; editor.conflict = false; editorState.set(editor); submit(ui); })}>Save my draft instead</Button>
                </View>
              </View>
            </View>
          </View>
          <SheetFooter>
            <View className={'w-full flex-col gap-3 text-[' + fontState.get().toString() + 'px]'}>
              <View hidden={editorState.get().confirm === ''} className="p-4 bg-muted flex-col gap-3" role="alert">
                <Text>
                  {editorState.get().confirm === 'delete' ? 'Delete this task? This change will sync to everyone.' : 'Discard your unsaved changes?'}
                </Text>
                <View className="flex-row gap-2">
                  <Button className={'shrink-0 text-[' + fontState.get().toString() + 'px]'} size="small" variant="danger" onPress={action(workspaceHandles(shared), (ui) => { if (editorState.get().confirm === 'delete') submit(ui, true); else closeEditor(ui, true); })}>Confirm</Button>
                  <Button className={'shrink-0 text-[' + fontState.get().toString() + 'px]'} size="small" variant="ghost" onPress={() => { const editor = editorState.get(); editor.confirm = ''; editorState.set(editor); }}>Keep editing</Button>
                </View>
              </View>
              <View className="w-full flex-row items-center gap-3">
                <Button className={'shrink-0 text-[' + fontState.get().toString() + 'px]'} size="small" variant="danger" outline={true} disabled={busyState.get() || !onlineState.get() || editorState.get().value.expectedVersion === 0} onPress={() => { const editor = editorState.get(); editor.confirm = 'delete'; editorState.set(editor); }}>Delete task</Button>
                <View className="grow" />
                <View hidden={onlineState.get()}>
                  <Text className="text-muted-foreground">Offline · your draft stays on this device</Text>
                </View>
                <Button className={'shrink-0 text-[' + fontState.get().toString() + 'px]'} size="small" variant="primary" loading={busyState.get()} disabled={!onlineState.get() || editorState.get().value.title.trim() === ''} onPress={action(workspaceHandles(shared), (ui) => { submit(ui); })}>Save task</Button>
              </View>
            </View>
          </SheetFooter>
        </SheetContent>
      </Sheet>
    </View>
  );
}

async function main(): Task<void> {
  setTheme('dark');
  const configured = unwrap(get('WORKLIST_API_URL')) ?? 'http://127.0.0.1:4317';
  const base = configured.endsWith('/') ? configured.slice(0, -1) : configured;
  const home = unwrap(homeDir());

  if (home === undefined) throw 'Worklist needs a home directory for its offline cache.';
  const root = unwrap(get('WORKLIST_DATA_DIR')) ?? join(home, '.tsrust', 'worklist', unwrap(toHex(sha256(unwrap(fromUtf8(base))))));
  unwrap(mkdirAll(root));
  const commands = new Channel<TaskCommand>(8);
  const updates = new Channel<TaskUpdate>(8);
  const replies = updates.sender();
  const view = App(commands.sender(), updates, join(root, 'draft.json'));
  const worker = taskService(base, 'sqlite:' + join(root, 'cache.sqlite') + '?mode=rwc', commands, replies).start();

  try {
    unwrap(run(view, 'Worklist', {
      titlebar: 'custom', width: 1180, height: 820, minWidth: 860, minHeight: 600
    }));
  } finally {
    worker.cancel();

    try {
      await worker;
    } catch (_) {}
  }
}
