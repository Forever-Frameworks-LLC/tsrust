import { toInt } from 'std:numeric';
import { basename } from 'std:path';
import {
  View,
  Text,
  Icon,
  Tooltip,
  List,
  ListItem,
  ListEmpty,
  Node,
  Signal,
  computed,
  translate,
  t,
  Button
} from 'std:ui';

import { SearchActionContext } from '../../lib/explorer/search-actions.tsr';
import { message, accessibleName } from '../../lib/localization.tsr';
import { openProblem, ProblemLocation } from '../../lib/workspace/problem-actions.tsr';
import { Problem, collectProblems } from '../../lib/workspace/problems.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { TerminalWorkspace, activateTerminal } from '../../lib/workspace/terminals.tsr';
import { Workspace } from '../../lib/workspace/workspace.tsr';
import { copyProblems, ProblemActionContext } from '../../services/problem-actions.tsr';


function revealProblem(
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>,
  terminals: Signal<TerminalWorkspace>,
  panel: Signal<string>,
  problems: Signal<Problem[]>,
  terminalError: Signal<string>,
  value: ReadOnly<string>
): void {
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

      if (problem.path === '' || problem.unversioned === true) return;
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
  const description = problem.code === 'unversioned-diagnostics' ? translate(problem.message) : clone(problem.message);
  const message = clone(description.split('\n').find((line) => line.trim() !== '') ?? description);
  const location = problem.path === '' ? clone(problem.source) : basename(problem.path) + (problem.line > 0? ':' + problem.line.toString() + ':' + problem.column.toString(): '');
  const details = description.slice(0, 8192);
  const attribution = problem.unversioned === true ? ' · ' + translate('Unversioned report') : '';
  const label = clone(location) + ' ' + clone(message) + clone(attribution);
  const metadata = problem.path === '' ? clone(problem.source) : location + ' [' + problem.source + (problem.code === problem.source ? '' : '/' + problem.code) + ']';
  const icon = problem.severity === 'warning' ? 'triangle-alert' : problem.severity === 'information' || problem.severity === 'hint' ? 'info' : 'circle-x';
  const iconClass = problem.severity === 'error' ? 'text-destructive' : 'text-muted-foreground';
  return (
    <ListItem value={index.toString()} accessibilityLabel={label} disabled={problem.unversioned === true || problem.path === '' && problem.terminal === undefined} className={rowClass.get()}>
      <Icon name={icon} size="sm" className={iconClass} />
      <Tooltip text={details} className="grow min-w-0">
        <Text className="grow min-w-0 truncate">
          {message}
        </Text>
      </Tooltip>
      <Text className="shrink-0 text-muted-foreground">
        {metadata + attribution}
      </Text>
    </ListItem>
  );
}

export function ProblemsPanel(
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>,
  terminals: Signal<TerminalWorkspace>,
  panel: Signal<string>,
  problems: Signal<Problem[]>,
  terminalError: Signal<string>,
  font: Signal<int>
): Node {
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
          revealProblem(
            workspace,
            recovery,
            terminals,
            panel,
            problems,
            terminalError,
            value
          );
        }}
        onConfirm={(value) => {
          revealProblem(
            workspace,
            recovery,
            terminals,
            panel,
            problems,
            terminalError,
            value
          );
        }}
      >
        {...collectProblems(workspace.get(), problems.get(), terminalError.get()).map((problem: ReadOnly<Problem>, index: int) => ProblemRow(problem, index, font))}
        <ListEmpty>
          <View />
        </ListEmpty>
      </List>
    </View>
  );
}

export function CopyProblems(workspace: Signal<Workspace>, problems: Signal<Problem[]>, terminalError: Signal<string>): Node {
  return (
    <Button
      icon="copy"
      size="xsmall"
      variant="ghost"
      tooltip={message('Copy all problems')}
      aria-label={accessibleName('Copy all problems')}
      onPress={() => {
        const context: ProblemActionContext = {
          workspace, problems, terminalError, allowClipboard: true
        };
        copyProblems(context);
      }}
    />
  );
}
