import { View, Text, Icon, Tooltip, List, ListItem, Node, Signal, computed, t } from 'std:ui';
import { basename } from 'std:path';
import { toInt } from 'std:numeric';
import { Workspace } from '../../lib/workspace/workspace.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { SearchActionContext } from '../../lib/explorer/search-actions.tsr';
import { openProblem, ProblemLocation } from '../../lib/workspace/problem-actions.tsr';
import { Problem, operationProblem } from '../../lib/workspace/problems.tsr';
import { TerminalWorkspace, activateTerminal } from '../../lib/workspace/terminals.tsr';

export function collectProblems(workspace: ReadOnly<Workspace>, problems: ReadOnly<Problem[]>, terminalError: ReadOnly<string>): Problem[] {
  const result = clone(problems);
  const error = clone(workspace.error);

  if (error !== '' && !result.some((problem) => problem.message === error)) result.push(operationProblem('studio', '', error));
  const terminal = clone(terminalError);

  if (terminal !== '' && !result.some((problem) => problem.message === terminal)) result.push(operationProblem('terminal', '', terminal));
  return result;
}

function revealProblem(workspace: Signal<Workspace>, recovery: Signal<Recovery>, terminals: Signal<TerminalWorkspace>, panel: Signal<string>, problems: Signal<Problem[]>, terminalError: Signal<string>, value: ReadOnly<string>): void {
  match (toInt(Number(value))) {
    Err(_) => {},
    Ok(index) => {
      const current = collectProblems(workspace.get(), problems.get(), terminalError.get());

      if (index < 0 || index >= current.length) return;
      const problem = current[index];

      if (problem.terminal !== undefined) {
        const next = terminals.get();

        match (activateTerminal(next, clone(problem.terminal))) {
          Ok(_) => {
            terminals.set(next);
            panel.set('terminal');
          },
          Err(_) => {}
        }

        return;
      }

      if (problem.path === '') return;
      const context: SearchActionContext = {
        workspace, recovery, allowNavigation: true, focusEditor: true
      };
      const location: ProblemLocation = {
        path: clone(problem.path), line: problem.line > 0 ? problem.line : 1, column: problem.column > 0 ? problem.column : 1
      };

      match (openProblem(context, location)) {
        Err(error) => {
          const next = workspace.get();
          next.error = clone(error);
          workspace.set(next);
        },
        Ok(_) => {}
      }
    }
  }
}

function ProblemRow(problem: ReadOnly<Problem>, index: int, font: Signal<int>): Node {
  const rowClass = computed<string>(() => 'py-1 border-b border-border rounded-none text-[' + font.get().toString() + 'px]');
  const message = clone(problem.message.split('\n').find((line) => line.trim() !== '') ?? problem.message);
  const location = problem.path === '' ? clone(problem.source) : basename(problem.path) + (problem.line > 0? ':' + problem.line.toString() + ':' + problem.column.toString(): '');
  const label = clone(location) + ' ' + clone(message);
  const details = problem.message.slice(0, 8192);
  const metadata = location + ' [' + problem.source + '/' + problem.code + ']';
  const icon = problem.severity === 'warning' ? 'triangle-alert' : 'circle-x';
  const iconClass = problem.severity === 'warning' ? 'text-muted-foreground' : 'text-destructive';
  return (
    <ListItem value={index.toString()} accessibilityLabel={label} disabled={problem.path === '' && problem.terminal === undefined} className={rowClass.get()}>
      <Icon name={icon} size="sm" className={iconClass} />
      <Tooltip text={details} className="grow min-w-0">
        <Text className="grow min-w-0 truncate">
          {message}
        </Text>
      </Tooltip>
      <Text className="shrink-0 text-muted-foreground">
        {metadata}
      </Text>
    </ListItem>
  );
}

export function ProblemsPanel(workspace: Signal<Workspace>, recovery: Signal<Recovery>, terminals: Signal<TerminalWorkspace>, panel: Signal<string>, problems: Signal<Problem[]>, terminalError: Signal<string>, font: Signal<int>): Node {
  const panelClass = computed<string>(() => 'flex-col size-full min-h-0 text-[' + font.get().toString() + 'px]');
  return (
    <View className={panelClass.get()}>
      <View hidden={collectProblems(workspace.get(), problems.get(), terminalError.get()).length !== 0} className="p-3">
        <Text className="text-muted-foreground">
          {t('No reported problems.')}
        </Text>
      </View>
      <List
        className="grow min-h-0"
        onSelect={(value) => {
          revealProblem(workspace, recovery, terminals, panel, problems, terminalError, value);
        }}
        onConfirm={(value) => {
          revealProblem(workspace, recovery, terminals, panel, problems, terminalError, value);
        }}
      >
        {...collectProblems(workspace.get(), problems.get(), terminalError.get()).map((problem: ReadOnly<Problem>, index: int) => ProblemRow(problem, index, font))}
      </List>
    </View>
  );
}
