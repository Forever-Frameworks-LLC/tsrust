import { readTextFile, writeTextFileAtomic } from 'std:fs';
import { JsonLimits } from 'std:json';
import {
  View,
  Text,
  Terminal,
  Button,
  Icon,
  Spinner,
  List,
  ListItem,
  ListItemSuffix,
  ListEmpty,
  DropdownMenu,
  MenuItem,
  Resizable,
  ResizablePanel,
  Node,
  Signal,
  computed,
  readTerminal,
  translate,
  translateWith,
  t
} from 'std:ui';

import { SearchActionContext } from '../../lib/explorer/search-actions.tsr';
import { message, accessibleName } from '../../lib/localization.tsr';
import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { dirty, reload } from '../../lib/workspace/documents.tsr';
import { ProblemLocation, openProblem } from '../../lib/workspace/problem-actions.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { TerminalWorkspace, TerminalSession, terminalExited, terminalLocation } from '../../lib/workspace/terminals.tsr';
import { Workspace } from '../../lib/workspace/workspace.tsr';
import { TerminalAuthority, TerminalArguments, invokeTerminalTool } from '../../services/terminal-actions.tsr';
import { TerminalSessionAuthority, invokeTerminalSessionTool } from '../../services/terminal-sessions.tsr';

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
  const toolName = 'terminal_' + (name === 'page-up' || name === 'page-down' ? 'scroll' : name);
  readTerminal(clone(id), (snapshot) => {
    const authority: TerminalAuthority = {
      sessionId: clone(id), allowRead: true, allowInput: true, allowNavigation: true, allowClipboard: true, allowTerminate: true
    };
    const args: TerminalArguments = {
      id: clone(id), generation: snapshot.generation ?? 0
    };

    if (name === 'page-up' || name === 'page-down') args.lines = name === 'page-up' ? (snapshot.rows ?? 1) : -(snapshot.rows ?? 1);
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
    const receipt = next.sessions[index].formatCompletion;

    if (receipt !== undefined) {
      match (writeTextFileAtomic(receipt, JSON.stringify({
        success: snapshot.exitCode === 0
      }))) {
        Ok(_) => {},
        Err(error) => {
          const files = workspace.get();
          files.error = 'Could not read the formatter result: ' + error.message;
          workspace.set(files);
        }
      }
    }

    const formatted = next.sessions[index].task === 'format' && snapshot.exitCode === 0;
    state.set(next);

    if (formatted) {
      const files = workspace.get();

      for (let current = 0; current < files.files.length; current += 1) {
        if (files.files[current].kind !== 'text' || dirty(files.files[current].document) || !(next.sessions[index].args ?? []).includes(files.files[current].document.path)) continue;

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

function SessionViewport(
  session: ReadOnly<TerminalSession>,
  directory: string,
  state: Signal<TerminalWorkspace>,
  font: Signal<int>,
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>
): Node {
  const id = clone(session.id);
  const ordinal = session.ordinal;
  const title = clone(session.title ?? '');
  const activeId = clone(id);
  const linkId = clone(id);
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
            const parsed = JSON.parse<ProblemLocation>(value, limits);
            const current = state.get().sessions.find((entry) => entry.id === linkId);

            if (current === undefined) return;
            let location = clone(parsed);

            match (terminalLocation(current, workspace.get(), parsed)) {
              Ok(mapped) => {
                location = clone(mapped);
              },
              Err(error) => {
                const next = workspace.get();
                next.error = clone(error);
                workspace.set(next);
                return;
              }
            }

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
  const outputId = clone(id);
  const upId = clone(id);
  const downId = clone(id);
  const restartId = clone(id);
  const killId = clone(id);
  const ordinal = session.ordinal;
  const title = clone(session.title ?? '');
  const name = computed<string>(() => clone(title) !== '' ? translate(clone(title)) : translateWith('Terminal {number}', {
    number: ordinal
  }));
  const task = session.task !== undefined;
  const statusId = clone(id);
  const exited = computed<boolean>(() => state.get().sessions.find((item) => item.id === statusId)?.exited ?? false);
  const failureId = clone(id);
  const failed = computed<boolean>(() => state.get().sessions.find((item) => item.id === failureId)?.failed ?? false);
  const textClass = computed<string>(() => 'grow min-w-0 truncate text-[' + font.get().toString() + 'px]');
  return (
    <ListItem value={clone(id)} accessibilityLabel={name.get()} className="py-0 pr-1.5 border-b border-border rounded-none">
      <View hidden={!task || exited.get()}>
        <Spinner size="small" />
      </View>
      <View hidden={task && !exited.get()}>
        <Icon name={task ? failed.get() ? 'circle-x' : 'circle-check' : exited.get() ? 'circle-stop' : 'terminal'} size="sm" className={failed.get() ? 'text-destructive' : ''} />
      </View>
      <Text className={textClass.get()}>
        {name.get()}
      </Text>
      <ListItemSuffix>
        <DropdownMenu fontSize={font.get()}>
          <Button icon="ellipsis" variant="ghost" size="xsmall" tooltip={message("Terminal actions")} aria-label={accessibleName("Terminal actions")} />
          <MenuItem
            label={message("Interrupt terminal")}
            disabled={exited.get()}
            onSelect={() => {
              humanTerminalAction(state, clone(interruptId), 'interrupt', error);
            }}
          />
          <MenuItem
            label={message("Copy terminal output")}
            onSelect={() => {
              humanTerminalAction(state, clone(outputId), 'copy-output', error);
            }}
          />
          <MenuItem
            label={message("Scroll up one page")}
            onSelect={() => {
              humanTerminalAction(state, clone(upId), 'page-up', error);
            }}
          />
          <MenuItem
            label={message("Scroll down one page")}
            onSelect={() => {
              humanTerminalAction(state, clone(downId), 'page-down', error);
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
            disabled={!exited.get() || task}
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

export function TerminalPanel(
  directory: string,
  font: Signal<int>,
  state: Signal<TerminalWorkspace>,
  error: Signal<string>,
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>
): Node {
  const panelClass = computed<string>(() => 'flex-col size-full min-h-0 text-[' + font.get().toString() + 'px]');
  const empty = computed<boolean>(() => state.get().sessions.length === 0);
  const viewport = (session: ReadOnly<TerminalSession>): Node => SessionViewport(
    session,
    clone(directory),
    state,
    font,
    workspace,
    recovery
  );
  const selected = computed<int>(() => {
    const current = state.get();
    return current.sessions.findIndex((session) => session.id === current.active);
  });
  return (
    <View className={panelClass.get()}>
      <Resizable className="size-full min-h-0 min-w-0">
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
          <View className="flex-col size-full min-h-0 bg-muted">
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
