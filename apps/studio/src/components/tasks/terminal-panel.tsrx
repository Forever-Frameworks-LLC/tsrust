import { View, Text, Terminal, Button, Icon, Spinner, List, ListItem, ListItemSuffix, ListEmpty, DropdownMenu, MenuItem, Resizable, ResizablePanel, Node, Signal, computed, readTerminal, translate, translateWith, t } from 'std:ui';
import { JsonLimits } from 'std:json';
import { SearchActionContext } from '../../lib/explorer/search-actions.tsr';
import { ProblemLocation, openProblem } from '../../lib/workspace/problem-actions.tsr';
import { readTextFile } from 'std:fs';
import { Workspace } from '../../lib/workspace/workspace.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { dirty, reload } from '../../lib/workspace/documents.tsr';
import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { TerminalAuthority, TerminalArguments, invokeTerminalTool } from '../../services/terminal-actions.tsr';
import { TerminalSessionAuthority, invokeTerminalSessionTool } from '../../services/terminal-sessions.tsr';
import { TerminalWorkspace, TerminalSession, terminalExited } from '../../lib/workspace/terminals.tsr';
import { message, accessibleName } from '../../lib/localization.tsr';

function humanSessionAction(state: Signal<TerminalWorkspace>, name: string, id: string, focus: boolean, error: Signal<string>): void {
  const authority: TerminalSessionAuthority = {
    allowRead: true, allowCreate: true, allowNavigate: true, allowTerminate: true
  };
  invokeTerminalSessionTool(state, authority, name, JSON.stringify({
    id: id, focus: focus
  }), (result) => {
    error.set(clone(result.error));
  });
}

function killTerminal(state: Signal<TerminalWorkspace>, id: string, error: Signal<string>): void {
  readTerminal(clone(id), (snapshot) => {
    const authority: TerminalSessionAuthority = {
      allowRead: true, allowCreate: true, allowNavigate: true, allowTerminate: true
    };
    invokeTerminalSessionTool(state, authority, 'terminal_sessions_remove', JSON.stringify({
      id: clone(id), generation: snapshot.generation ?? 0
    }), (result) => {
      error.set(clone(result.error));
    });
  });
}

function humanTerminalAction(state: Signal<TerminalWorkspace>, id: string, name: string, error: Signal<string>): void {
  const restart = name === 'restart';
  const toolName = 'terminal_' + name;
  readTerminal(clone(id), (snapshot) => {
    const authority: TerminalAuthority = {
      sessionId: clone(id), allowRead: true, allowInput: true, allowNavigation: true, allowClipboard: true, allowTerminate: true
    };
    const args: TerminalArguments = {
      id: clone(id), generation: snapshot.generation ?? 0
    };
    const changedId = clone(id);
    invokeTerminalTool(authority, clone(toolName), JSON.stringify(args), (result) => {
      error.set(clone(result.error));

      if (result.ok && restart) {
        const next = state.get();
        terminalExited(next, clone(changedId), false);
        state.set(next);
      }
    });
  });
}

function sessionEnded(state: Signal<TerminalWorkspace>, id: string, workspace: Signal<Workspace>, recovery: Signal<Recovery>): void {
  readTerminal(clone(id), (snapshot) => {
    if (!snapshot.found || !(snapshot.exited ?? false)) return;
    const next = state.get();
    const index = next.sessions.findIndex((session) => session.id === id);

    if (index < 0) return;
    terminalExited(next, clone(id), true);
    next.sessions[index].failed = (snapshot.error ?? '') !== '' || (next.sessions[index].task !== undefined ? snapshot.exitCode !== 0 : (snapshot.exitCode ?? 0) !== 0);
    const formatted = next.sessions[index].task === 'format' && snapshot.exitCode === 0;
    state.set(next);

    if (formatted) {
      const files = workspace.get();

      for (let current = 0; current < files.files.length; current += 1) {
        if (files.files[current].kind !== 'text' || dirty(files.files[current].document)) continue;

        match (readTextFile(files.files[current].document.path, 16 * 1024 * 1024)) {
          Ok(text) => {
            reload(files.files[current].document, clone(text));
          },
          Err(error) => {
            files.error = clone(error.message);
          }
        }
      }

      publishWorkspace(files, workspace, recovery);
    }
  });
}

function SessionViewport(session: ReadOnly<TerminalSession>, directory: string, state: Signal<TerminalWorkspace>, font: Signal<int>, workspace: Signal<Workspace>, recovery: Signal<Recovery>): Node {
  const id = clone(session.id);
  const ordinal = session.ordinal;
  const title = clone(session.title ?? '');
  const activeId = clone(id);
  const hidden = computed<boolean>(() => state.get().active !== clone(activeId));
  const name = computed<string>(() => clone(title) !== '' ? translate(clone(title)) : translateWith('Terminal {number}', {
    number: ordinal
  }));
  return (
    <View id={clone(id)} hidden={hidden.get()} className="size-full min-w-0 min-h-0">
      <Terminal
        id={clone(id)}
        directory={clone(directory)}
        program={clone(session.program)}
        args={clone(session.args)}
        fontSize={font.get()}
        onOpenLink={(value) => {
          const limits: JsonLimits = {
            maxBytes: 16384, maxDepth: 4, maxEntries: 8, maxNodes: 16
          };
          const context: SearchActionContext = {
            workspace, recovery, allowNavigation: true, focusEditor: true
          };

          try {
            const location = JSON.parse<ProblemLocation>(value, limits);

            match (openProblem(context, location)) {
              Err(error) => {
                const next = workspace.get();
                next.error = clone(error);
                workspace.set(next);
              },
              Ok(_) => {}
            }
          } catch (_) {
            const next = workspace.get();
            next.error = 'Invalid terminal file location.';
            workspace.set(next);
          }
        }}
        onExit={() => {
          sessionEnded(state, clone(id), workspace, recovery);
        }}
        accessibilityLabel={name.get()}
        className="size-full"
      />
    </View>
  );
}

function SessionItem(session: ReadOnly<TerminalSession>, state: Signal<TerminalWorkspace>, font: Signal<int>, error: Signal<string>): Node {
  const id = clone(session.id);
  const interruptId = clone(id);
  const copyId = clone(id);
  const restartId = clone(id);
  const killId = clone(id);
  const ordinal = session.ordinal;
  const title = clone(session.title ?? '');
  const name = computed<string>(() => clone(title) !== '' ? translate(clone(title)) : translateWith('Terminal {number}', {
    number: ordinal
  }));
  const textClass = computed<string>(() => 'grow min-w-0 truncate text-[' + font.get().toString() + 'px]');
  return (
    <ListItem value={clone(id)} accessibilityLabel={name.get()} className="py-0 border-b border-border rounded-none">
      <View hidden={session.task === undefined || session.exited}>
        <Spinner size="small" />
      </View>
      <View hidden={session.task !== undefined && !session.exited}>
        <Icon name={session.task !== undefined ? (session.failed ?? false) ? 'circle-x' : 'circle-check' : session.exited ? 'circle-stop' : 'terminal'} size="sm" className={(session.failed ?? false) ? 'text-destructive' : ''} />
      </View>
      <Text className={textClass.get()}>
        {name.get()}
      </Text>
      <ListItemSuffix>
        <DropdownMenu>
          <Button icon="ellipsis" variant="ghost" size="xsmall" tooltip={message("Terminal actions")} aria-label={accessibleName("Terminal actions")} />
          <MenuItem
            label={message("Interrupt terminal")}
            disabled={session.exited}
            onSelect={() => {
              humanTerminalAction(state, clone(interruptId), 'interrupt', error);
            }}
          />
          <MenuItem
            label={message("Copy terminal selection")}
            onSelect={() => {
              humanTerminalAction(state, clone(copyId), 'copy', error);
            }}
          />
          <MenuItem
            label={message("Restart exited shell")}
            disabled={!session.exited || session.task !== undefined}
            onSelect={() => {
              humanTerminalAction(state, clone(restartId), 'restart', error);
            }}
          />
        </DropdownMenu>
        <Button
          icon="trash"
          variant="ghost"
          size="xsmall"
          tooltip={message("Kill terminal")}
          aria-label={accessibleName("Kill terminal")}
          onPress={() => {
            killTerminal(state, clone(killId), error);
          }}
        />
      </ListItemSuffix>
    </ListItem>
  );
}

export function NewTerminal(state: Signal<TerminalWorkspace>, error: Signal<string>): Node {
  return (
    <Button
      icon="plus"
      variant="ghost"
      size="xsmall"
      tooltip={message("New terminal")}
      aria-label={accessibleName("New terminal")}
      onPress={() => {
        humanSessionAction(state, 'terminal_sessions_create', '', true, error);
      }}
    />
  );
}

export function TerminalPanel(directory: string, font: Signal<int>, state: Signal<TerminalWorkspace>, error: Signal<string>, workspace: Signal<Workspace>, recovery: Signal<Recovery>): Node {
  const panelClass = computed<string>(() => 'flex-col size-full min-h-0 text-[' + font.get().toString() + 'px]');
  const empty = computed<boolean>(() => state.get().sessions.length === 0);
  const viewport = (session: ReadOnly<TerminalSession>): Node => SessionViewport(session, clone(directory), state, font, workspace, recovery);
  const selected = computed<int>(() => {
    const current = state.get();
    return current.sessions.findIndex((session) => session.id === current.active);
  });
  return (
    <View className={panelClass.get()}>
      <Resizable className="grow min-h-0 min-w-0">
        <ResizablePanel minSize={160}>
          <View className="size-full min-h-0">
            <View hidden={!empty.get()} className="p-2">
              <Text className="text-muted-foreground">
                {t("No terminal sessions")}
              </Text>
            </View>
            {...state.get().sessions.map(viewport)}
          </View>
        </ResizablePanel>
        <ResizablePanel size={180} minSize={96} maxSize={360}>
          <View className="flex-col size-full min-h-0 border-l border-border bg-muted">
            <List
              selected={selected.get()}
              size="small"
              className="grow min-h-0"
              onSelect={(id) => {
                humanSessionAction(state, 'terminal_sessions_activate', clone(id), false, error);
              }}
              onConfirm={(id) => {
                humanSessionAction(state, 'terminal_sessions_activate', clone(id), true, error);
              }}
            >
              {...state.get().sessions.map((session: ReadOnly<TerminalSession>) => SessionItem(session, state, font, error))}
              <ListEmpty>
                <View />
              </ListEmpty>
            </List>
          </View>
        </ResizablePanel>
      </Resizable>
    </View>
  );
}
