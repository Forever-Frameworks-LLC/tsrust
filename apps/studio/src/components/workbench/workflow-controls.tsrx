import { writeTextFileAtomic } from 'std:fs';
import { Channel, Sender } from 'std:futures';
import {
  View,
  Button,
  DropdownMenu,
  MenuItem,
  Shortcut,
  Ticker,
  Node,
  Signal,
  useId,
  computed,
  signal
} from 'std:ui';

import { message, accessibleName } from '../../lib/localization.tsr';
import { EditorPolicy, projectEditorPolicy } from '../../lib/settings/formatting.tsr';
import { StudioConfig } from '../../lib/settings/settings.tsr';
import { saveTarget, publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import {
  QueuedFormatting,
  FormattingTarget,
  FormattingAuthority,
  FormattingRequest,
  FormattingResult,
  prepareFormatting,
  applyFormatting,
  activeSourceDocument,
  canFormat
} from '../../lib/workspace/formatting-actions.tsr';
import { Problem, operationProblem, replaceProblems } from '../../lib/workspace/problems.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { TerminalWorkspace, addTerminal } from '../../lib/workspace/terminals.tsr';
import { Workspace } from '../../lib/workspace/workspace.tsr';
import { runProjectTask } from '../../services/project-tasks.tsr';
import { SaveActionContext, queueSaves, unsavedTargets } from '../../services/save-actions.tsr';
import { beginCheck } from '../tasks/check-panel.tsrx';

function requestSaveAll(workspace: Signal<Workspace>, queue: Signal<QueuedFormatting[]>): void {
  const context: SaveActionContext = {
    workspace, queue, allowSave: true
  };

  match (queueSaves(context, unsavedTargets(context.workspace.get()))) {
    Ok(_) => {},
    Err(error) => {
      const next = context.workspace.get();
      next.error = clone(error);
      context.workspace.set(next);
    }
  }
}

export function requestAction(
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>,
  settings: Signal<StudioConfig>,
  requests: ReadOnly<Sender<FormattingRequest>>,
  checks: ReadOnly<Sender<string>>,
  busy: Signal<boolean>,
  checking: Signal<boolean>,
  output: Signal<string>,
  problems: Signal<Problem[]>,
  saveCheckPending: Signal<boolean>,
  mode: string,
  queuedTarget: FormattingTarget | undefined = undefined
): void {
  const next = workspace.get();
  const targetPath = queuedTarget === undefined ? clone(next.active) : clone(queuedTarget.path);

  if (busy.get()) {
    if (mode === 'focus') return;
    next.error = 'A formatting request is already running.';
    workspace.set(next);
    return;
  }

  const index = next.files.findIndex((file) => file.document.path === targetPath);

  if (index < 0) return;
  const target: FormattingTarget = {
    path: clone(targetPath), session: queuedTarget === undefined ? clone(next.files[index].document.session) : clone(queuedTarget.session), revision: queuedTarget === undefined ? next.files[index].document.revision : queuedTarget.revision
  };

  const config = settings.get();
  const defaults: EditorPolicy = {
    formatOnSave: config.formatOnSave, checkOnSave: config.checkOnSave, formatOnFocusChange: config.formatOnFocusChange, checkOnFocusChange: config.checkOnFocusChange, format: clone(config.format)
  };

  match (projectEditorPolicy(next.root, defaults)) {
    Err(error) => {
      next.error = clone(error);
      workspace.set(next);
    },
    Ok(policy) => {
      if (mode === 'focus' && (!policy.formatOnFocusChange || !canFormat(targetPath))) {
        if (policy.checkOnFocusChange) beginCheck(checks, checking, output, false);
        return;
      }

      if (mode === 'save' && (!policy.formatOnSave || !canFormat(targetPath))) {
        const saved = saveTarget(next, target);
        const failures: Problem[] = [];
        let success = false;

        match (saved) {
          Ok(_) => {
            success = true;
          },
          Err(error) => {
            if (next.files[index].document.saveConflict === undefined) failures.push(operationProblem('save', clone(targetPath), clone(error)));
          }
        }

        problems.set(replaceProblems(problems.get(), 'save', failures, clone(targetPath)));
        publishWorkspace(next, workspace, recovery);

        if (success && policy.checkOnSave) saveCheckPending.set(true);
        return;
      }

      const authority: FormattingAuthority = {
        allowPropose: true, allowApply: true, allowSave: true
      };

      match (prepareFormatting(next, policy, authority, target, mode)) {
        Err(error) => {
          next.error = clone(error);
          workspace.set(next);
        },
        Ok(request) => {
          match (requests.trySend(request)) {
            Ok(_) => {
              next.error = '';
              workspace.set(next);
              busy.set(true);
            },
            Err(error) => {
              next.error = clone(error.code);
              workspace.set(next);
            }
          }
        }
      }
    }
  }
}

export function WorkflowControls(
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>,
  settings: Signal<StudioConfig>,
  panel: Signal<string>,
  requests: ReadOnly<Channel<FormattingRequest>>,
  results: Channel<FormattingResult>,
  checks: ReadOnly<Channel<string>>,
  checking: Signal<boolean>,
  output: Signal<string>,
  problems: Signal<Problem[]>,
  terminals: Signal<TerminalWorkspace>,
  terminalError: Signal<string>,
  program: string,
  busy: Signal<boolean>,
  focusQueue: Signal<QueuedFormatting[]>,
  font: Signal<int>
): Node {
  const saveCheckPending = signal(false);
  const available = computed<boolean>(() => activeSourceDocument(workspace.get()));
  const buttonRequests = requests.sender();
  const shortcutRequests = requests.sender();
  const buttonProgram = clone(program);
  const formatProgram = clone(program);
  const formatButtonChecks = checks.sender();
  const formatShortcutChecks = checks.sender();
  const completionChecks = checks.sender();
  const focusRequests = requests.sender();
  const focusChecks = checks.sender();
  return (
    <View className="flex-row items-center gap-1">
      <Button
        icon="terminal"
        variant="ghost"
        size="xsmall"
        className="size-5 p-0"
        tooltip={message("Toggle terminal")}
        aria-label={accessibleName("Toggle terminal")}
        onPress={() => {
          panel.set(panel.get() === 'terminal' ? '' : 'terminal');
        }}
      />
      <DropdownMenu fontSize={font.get()}>
        <Button icon="braces" className="size-5 p-0" variant="ghost" size="xsmall" loading={busy.get()} tooltip={message('Format')} aria-label={accessibleName('Format')} />
        <MenuItem label={message('Document')} icon="file-code" command="format-document" disabled={!available.get() || busy.get()} />
        <MenuItem label={message('Project')} icon="folder" command="format-project" disabled={busy.get()} />
      </DropdownMenu>
      <DropdownMenu fontSize={font.get()}>
        <Button icon="circle-check" className="size-5 p-0" variant="ghost" size="xsmall" tooltip={message('Check')} aria-label={accessibleName('Check')} />
        <MenuItem label={message('Document')} icon="file-code" command="check-document" disabled={!available.get()} />
        <MenuItem label={message('Project')} icon="folder" command="check-project" />
      </DropdownMenu>
      <Shortcut
        command="check-document"
        onTrigger={() => {
          runProjectTask(
            workspace,
            settings,
            terminals,
            panel,
            terminalError,
            clone(buttonProgram),
            'check-document'
          );
        }}
      />
      <Shortcut
        command="format-document"
        keys="alt-shift-f"
        onTrigger={() => {
          requestAction(
            workspace,
            recovery,
            settings,
            shortcutRequests,
            formatShortcutChecks,
            busy,
            checking,
            output,
            problems,
            saveCheckPending,
            'format'
          );
        }}
      />
      <Shortcut
        command="save-all"
        keys="secondary-s"
        onTrigger={() => {
          requestSaveAll(workspace, focusQueue);
        }}
      />
      <Shortcut
        command="check-project"
        keys="secondary-shift-b"
        onTrigger={() => {
          runProjectTask(
            workspace,
            settings,
            terminals,
            panel,
            terminalError,
            clone(program),
            'check'
          );
        }}
      />
      <Shortcut
        command="format-project"
        keys="secondary-alt-shift-f"
        onTrigger={() => {
          runProjectTask(
            workspace,
            settings,
            terminals,
            panel,
            terminalError,
            clone(formatProgram),
            'format',
            focusQueue
          );
        }}
      />
      <Ticker
        interval={100}
        running={busy.get() || focusQueue.get().length > 0 || saveCheckPending.get()}
        onTick={(_frame) => {
          if (!busy.get()) {
            const pending = focusQueue.get();

            if (pending.length > 0) {
              const entry = pending.shift();
              focusQueue.set(pending);

              if (entry !== undefined) requestAction(
                workspace,
                recovery,
                settings,
                focusRequests,
                focusChecks,
                busy,
                checking,
                output,
                problems,
                saveCheckPending,
                clone(entry.mode),
                clone(entry.target)
              );
            }
          }

          if (!busy.get()) {
            if (saveCheckPending.get() && !checking.get() && !focusQueue.get().some((entry) => entry.mode === 'save')) {
              saveCheckPending.set(false);
              beginCheck(focusChecks, checking, output, false);
            }

            return;
          }

          match (results.tryReceive()) {
            Err(error) => {
              if (error.code !== 'empty') {
                const next = workspace.get();
                next.error = clone(error.code);
                workspace.set(next);
                busy.set(false);
              }
            },
            Ok(result) => {
              const task = clone(result.task);

              if (task !== undefined) {
                const sessions = terminals.get();
                const id = useId();

                match (addTerminal(sessions, clone(id))) {
                  Ok(_) => {
                    const index = sessions.sessions.length - 1;
                    sessions.sessions[index].title = 'Format document';
                    sessions.sessions[index].program = clone(task.program);
                    sessions.sessions[index].args = clone(task.args);
                    sessions.sessions[index].task = 'format-document';
                    sessions.sessions[index].formatCompletion = clone(task.completion);
                    sessions.sessions[index].formatTarget = {
                      path: clone(result.request.document.path), session: clone(result.request.document.session), revision: result.request.document.revision
                    };
                    terminals.set(sessions);
                    panel.set('terminal');
                  },
                  Err(error) => {
                    terminalError.set(clone(error));
                    writeTextFileAtomic(task.completion, JSON.stringify({
                      success: false
                    }));
                  }
                }

                return;
              }

              const next = workspace.get();
              next.error = '';
              let check = false;

              match (applyFormatting(next, result)) {
                Ok(needed) => {
                  check = needed;
                  problems.set(replaceProblems(problems.get(), 'formatter', [], clone(result.request.document.path)));

                  if (result.request.mode === 'save') problems.set(replaceProblems(problems.get(), 'save', [], clone(result.request.document.path)));
                },
                Err(error) => {
                  const conflicted = next.files.some((file) => file.document.path === result.request.document.path && file.document.saveConflict !== undefined);
                  problems.set(replaceProblems(problems.get(), 'formatter', conflicted ? [] : [operationProblem('formatter', clone(result.request.document.path), clone(error))], clone(result.request.document.path)));

                  if (result.request.mode === 'format' && result.error === '') panel.set('problems');
                }
              }

              publishWorkspace(next, workspace, recovery);
              busy.set(false);

              if (check) {
                if (result.request.mode === 'save') saveCheckPending.set(true);
                else beginCheck(completionChecks, checking, output, false);
              }
            }
          }
        }}
      />
    </View>
  );
}
