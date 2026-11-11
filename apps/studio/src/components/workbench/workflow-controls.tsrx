import { View, Button, Shortcut, Ticker, Node, Signal, computed } from 'std:ui';
import { Channel, Sender } from 'std:futures';
import { Workspace } from '../../lib/workspace/workspace.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { saveActive, publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { StudioConfig } from '../../lib/settings/settings.tsr';
import { EditorPolicy, projectEditorPolicy } from '../../lib/settings/formatting.tsr';
import { QueuedFormatting, FormattingTarget, FormattingAuthority, FormattingRequest, FormattingResult, prepareFormatting, applyFormatting, canFormat } from '../../lib/workspace/formatting-actions.tsr';
import { beginCheck } from '../tasks/check-panel.tsrx';
import { message, accessibleName } from '../../lib/localization.tsr';
import { Problem, operationProblem, replaceProblems } from '../../lib/workspace/problems.tsr';
import { TerminalWorkspace } from '../../lib/workspace/terminals.tsr';
import { runProjectTask } from '../../services/project-tasks.tsr';

export function requestAction(workspace: Signal<Workspace>, recovery: Signal<Recovery>, settings: Signal<StudioConfig>, requests: ReadOnly<Sender<FormattingRequest>>, checks: ReadOnly<Sender<string>>, busy: Signal<boolean>, checking: Signal<boolean>, output: Signal<string>, mode: string, queuedTarget: FormattingTarget | undefined = undefined): void {
  const next = workspace.get();
  const targetPath = queuedTarget === undefined ? clone(next.active) : clone(queuedTarget.path);

  if (busy.get()) {
    if (mode === 'focus') return;
    next.error = 'A formatting request is already running.';
    workspace.set(next);
    return;
  }

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

      if (mode === 'save' && (!policy.formatOnSave || !canFormat(next.active))) {
        saveActive(workspace, recovery);

        if (workspace.get().error === '' && policy.checkOnSave) beginCheck(checks, checking, output, false);
        return;
      }

      const index = next.files.findIndex((file) => file.document.path === targetPath);

      if (index < 0) return;
      const target: FormattingTarget = {
        path: clone(targetPath), session: queuedTarget === undefined ? clone(next.files[index].document.session) : clone(queuedTarget.session), revision: queuedTarget === undefined ? next.files[index].document.revision : queuedTarget.revision
      };
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

export function WorkflowControls(workspace: Signal<Workspace>, recovery: Signal<Recovery>, settings: Signal<StudioConfig>, panel: Signal<string>, requests: ReadOnly<Channel<FormattingRequest>>, results: Channel<FormattingResult>, checks: ReadOnly<Channel<string>>, checking: Signal<boolean>, output: Signal<string>, problems: Signal<Problem[]>, terminals: Signal<TerminalWorkspace>, terminalError: Signal<string>, program: string, busy: Signal<boolean>, focusQueue: Signal<QueuedFormatting[]>): Node {
  const available = computed<boolean>(() => canFormat(workspace.get().active));
  const buttonRequests = requests.sender();
  const shortcutRequests = requests.sender();
  const saveRequests = requests.sender();
  const buttonProgram = clone(program);
  const formatProgram = clone(program);
  const formatButtonChecks = checks.sender();
  const formatShortcutChecks = checks.sender();
  const saveChecks = checks.sender();
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
      <Button
        icon="braces"
        className="size-5 p-0"
        variant="ghost"
        size="xsmall"
        disabled={!available.get() || busy.get()}
        loading={busy.get()}
        tooltip={message("Format document")}
        tooltipCommand="format-document"
        aria-label={accessibleName("Format document")}
        onPress={() => {
          requestAction(workspace, recovery, settings, buttonRequests, formatButtonChecks, busy, checking, output, 'format');
        }}
      />
      <Button
        icon="circle-check"
        className="size-5 p-0"
        variant="ghost"
        size="xsmall"
        disabled={checking.get()}
        tooltip={message("Check project")}
        tooltipCommand="check-project"
        aria-label={accessibleName("Check project")}
        onPress={() => {
          runProjectTask(workspace, settings, terminals, panel, terminalError, clone(buttonProgram), 'check');
        }}
      />
      <Shortcut
        command="format-document"
        keys="alt-shift-f"
        onTrigger={() => {
          requestAction(workspace, recovery, settings, shortcutRequests, formatShortcutChecks, busy, checking, output, 'format');
        }}
      />
      <Shortcut
        command="save-file"
        keys="secondary-s"
        onTrigger={() => {
          requestAction(workspace, recovery, settings, saveRequests, saveChecks, busy, checking, output, 'save');
        }}
      />
      <Shortcut
        command="check-project"
        keys="secondary-shift-b"
        onTrigger={() => {
          runProjectTask(workspace, settings, terminals, panel, terminalError, clone(program), 'check');
        }}
      />
      <Shortcut
        command="format-project"
        keys="secondary-alt-shift-f"
        onTrigger={() => {
          runProjectTask(workspace, settings, terminals, panel, terminalError, clone(formatProgram), 'format', focusQueue);
        }}
      />
      <Ticker
        interval={100}
        running={busy.get() || focusQueue.get().length > 0}
        onTick={(_frame) => {
          if (!busy.get()) {
            const pending = focusQueue.get();

            if (pending.length > 0) {
              const entry = pending.shift();
              focusQueue.set(pending);

              if (entry !== undefined) requestAction(workspace, recovery, settings, focusRequests, focusChecks, busy, checking, output, clone(entry.mode), clone(entry.target));
            }
          }

          if (!busy.get()) return;

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
              const next = workspace.get();
              next.error = '';
              let check = false;

              match (applyFormatting(next, result)) {
                Ok(needed) => {
                  check = needed;
                  problems.set(replaceProblems(problems.get(), 'formatter', []));
                },
                Err(error) => {
                  const conflicted = next.files.some((file) => file.document.path === result.request.document.path && file.document.saveConflict !== undefined);
                  problems.set(replaceProblems(problems.get(), 'formatter', conflicted ? [] : [operationProblem('formatter', clone(result.request.document.path), clone(error))]));

                  if (result.request.mode === 'format') panel.set('problems');
                }
              }

              publishWorkspace(next, workspace, recovery);
              busy.set(false);

              if (check) beginCheck(completionChecks, checking, output, false);
            }
          }
        }}
      />
    </View>
  );
}
