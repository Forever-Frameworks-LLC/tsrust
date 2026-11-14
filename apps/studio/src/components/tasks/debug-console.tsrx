import { Channel, Sender } from 'std:futures';
import {
  Button,
  CodeEditor,
  Input,
  InputGroup,
  Text,
  Node,
  Signal,
  Ticker,
  View,
  computed,
  readInput,
  pickFilePaths,
  signal,
  t,
  translate
} from 'std:ui';

import { SearchActionContext, fileSearchLabel } from '../../lib/explorer/search-actions.tsr';
import { DiagnosticPosition, documentUri } from '../../lib/language/diagnostics.tsr';
import { openLanguageLocation } from '../../lib/language/navigation.tsr';
import { message, accessibleName } from '../../lib/localization.tsr';
import { DebugCommand, DebugOutput, SourceFrame, UnresolvedBreakpoint } from '../../services/debug/session.tsr';

import { Workspace, SourceBreakpoint } from '../../lib/workspace/workspace.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';

export function toggleCursorBreakpoint(workspace: Signal<Workspace>, recovery: Signal<Recovery>, listed: Signal<SourceBreakpoint[]>, running: boolean): void {
  const active = workspace.get().active;

  if (running || active === '') return;
  const path = clone(active);
  readInput(clone(path), (field) => {
    if (!field.found) return;
    const caret = field.selectionReversed === true ? field.selectionStart : field.selectionEnd;

    if (caret === undefined || field.value === undefined) return;
    toggleLineBreakpoint(workspace, recovery, listed, path, field.value.slice(0, caret).split('\n').length);
  });
}

export function toggleLineBreakpoint(workspace: Signal<Workspace>, recovery: Signal<Recovery>, listed: Signal<SourceBreakpoint[]>, path: ReadOnly<string>, line: int): void {
  const next = workspace.get();
  const points = clone(next.sourceBreakpoints) ?? [];
  const index = points.findIndex((item) => item.path === path && item.line === line);

  if (index < 0) points.push({
    path: clone(path), line
  });
  else points.splice(index, 1);
  listed.set(clone(points));
  next.sourceBreakpoints = points;
  publishWorkspace(next, workspace, recovery);
}

export function sendDebugControl(sender: ReadOnly<Sender<DebugCommand>>, stopped: Signal<boolean>, output: Signal<string>, action: string): void {
  if (action !== 'stop' && action !== 'pause' && !stopped.get()) return;

  match (sender.trySend({
    action
  })) {
    Ok(_) => {
      stopped.set(false);
    },
    Err(error) => {
      output.set(output.get() + error.code);
    }
  }
}

function DebugControl(
  requests: ReadOnly<Channel<DebugCommand>>,
  stopped: Signal<boolean>,
  output: Signal<string>,
  action: string,
  label: string,
  icon: string
): Node {
  const sender = requests.sender();
  const command = 'debug-' + action;
  return (
    <Button
      icon={icon}
      aria-label={accessibleName(clone(label))}
      variant="ghost"
      size="xsmall"
      tooltip={message(clone(label))}
      tooltipCommand={command}
      onPress={() => {
        sendDebugControl(sender, stopped, output, clone(action));
      }}
    />
  );
}

function openDebugLocation(navigation: ReadOnly<SearchActionContext>, path: ReadOnly<string>, line: int, column: int, output: Signal<string>): void {
  const point: DiagnosticPosition = {
    line: line - 1, character: column - 1
  };

  match (openLanguageLocation(navigation, {
    root: clone(navigation.allowedRoot ?? ''), uri: documentUri(path), selection: {
      start: clone(point), end: point
    }
  })) {
    Ok(_) => {},
    Err(error) => {
      output.set(output.get() + error + '\n');
    }
  }
}

function BreakpointRow(
  target: SourceBreakpoint,
  unresolved: Signal<UnresolvedBreakpoint[]>,
  listed: Signal<SourceBreakpoint[]>,
  root: ReadOnly<string>,
  running: Signal<boolean>,
  font: Signal<int>,
  navigation: SearchActionContext,
  output: Signal<string>
): Node {
  const label = fileSearchLabel(root, target.path) + ':' + target.line.toString();
  const rowWorkspace = navigation.workspace.handle();
  const rowRecovery = navigation.recovery.handle();
  const file = clone(target.path);
  const line = target.line;
  const markerPath = clone(file);
  const rowLabel = clone(label);
  const description = computed<string>(() => {
    const point = unresolved.get().find((item) => item.path === markerPath && item.line === line);
    return point === undefined ? clone(rowLabel) : translate(clone(point.reason)) + ' · ' + clone(rowLabel);
  });
  return (
    <View className="flex-row items-center w-full min-w-0 border-b border-border hover:bg-accent">
      <Button
        className={'grow min-w-0 justify-start px-2 py-[6px] rounded-none text-[' + font.get().toString() + 'px]'}
        variant="ghost"
        size="small"
        icon={description.get() !== clone(label) ? 'circle-alert' : undefined}
        tooltip={description.get()}
        aria-label={description.get()}
        onPress={() => {
          openDebugLocation(navigation, file, line, 1, output);
        }}
      >
        <Text className={'truncate text-[' + font.get().toString() + 'px]'}>
          {label}
        </Text>
      </Button>
      <Button
        icon="x"
        size="xsmall"
        variant="ghost"
        disabled={running.get()}
        tooltip={message('Remove breakpoint')}
        aria-label={translate('Remove breakpoint') + ' ' + fileSearchLabel(root, target.path) + ':' + target.line.toString()}
        onPress={() => {
          const next = rowWorkspace.get();
          const points = clone(next.sourceBreakpoints) ?? [];
          const index = points.findIndex((item) => item.path === target.path && item.line === target.line);

          if (index >= 0) points.splice(index, 1);
          listed.set(clone(points));
          next.sourceBreakpoints = points;
          publishWorkspace(next, rowWorkspace, rowRecovery);
        }}
      />
    </View>
  );
}

function StackFrameRow(
  frame: SourceFrame,
  root: ReadOnly<string>,
  stopId: int,
  selected: Signal<int>,
  requests: Sender<DebugCommand>,
  stopped: Signal<boolean>,
  output: Signal<string>,
  font: Signal<int>
): Node {
  const frameId = frame.id;
  const location = frame.location;
  const label = '#' + frame.depth.toString() + ' ' + frame.name + (location === undefined ? '' : '  ' + fileSearchLabel(root, location.path) + ':' + location.line.toString());
  return (
    <Button
      className={'w-full min-w-0 justify-start px-2 py-[6px] rounded-none border-b border-border hover:bg-accent text-[' + font.get().toString() + 'px]' + (selected.get() === frameId ? ' bg-accent text-accent-foreground' : '')}
      variant="ghost"
      size="small"
      selected={selected.get() === frameId}
      tooltip={clone(label)}
      aria-label={clone(label)}
      onPress={() => {
        match (requests.trySend({
          action: 'selectFrame', frameId, stopId
        })) {
          Ok(_) => {
            stopped.set(false);
          },
          Err(reason) => {
            output.set(output.get() + reason.code);
          }
        }
      }}
    >
      <Text className={'truncate text-[' + font.get().toString() + 'px]'}>
        {label}
      </Text>
    </Button>
  );
}

export function chooseDebugTarget(chosen: Signal<string>): void {
  const target = chosen.handle();
  pickFilePaths((paths) => {
    if (paths.length > 0) target.set(clone(paths[0]));
  }, {
    multiple: false
  });
}

function InspectionHeading(label: string, open: Signal<boolean>, font: Signal<int>): Node {
  return (
    <Button
      className={'w-full min-w-0 justify-start rounded-none px-2 text-muted-foreground min-h-[' + (font.get() + 12).toString() + 'px]'}
      size="xsmall"
      variant="ghost"
      icon={open.get() ? 'chevron-down' : 'chevron-right'}
      aria-label={accessibleName(clone(label))}
      aria-expanded={open.get()}
      onPress={() => {
        open.set(!open.get());
      }}
    >
      <Text className={'truncate text-[' + font.get().toString() + 'px]'}>
        {t(clone(label))}
      </Text>
    </Button>
  );
}

export function DebugSidebar(
  requests: ReadOnly<Channel<DebugCommand>>,
  replies: Channel<DebugOutput>,
  font: Signal<int>,
  output: Signal<string>,
  panel: Signal<string>,
  navigation: SearchActionContext,
  listed: Signal<SourceBreakpoint[]>,
  running: Signal<boolean>,
  stopped: Signal<boolean>,
  chosen: Signal<string>,
  lastTarget: Signal<string>
): Node {
  const sender = requests.sender();
  const stopOnEntry = signal(false);
  const workspace = navigation.workspace.handle();
  const selectedWorkspace = workspace.handle();
  const root = clone(workspace.get().root);
  const frameRoot = clone(root);
  const frameStopped = stopped.handle();
  const frameOutput = output.handle();
  const frameFont = font.handle();
  const breakpoint = signal('');
  const watchSender = requests.sender();
  const watch = signal('');
  const watchValue = signal('');
  const frames = signal<SourceFrame[]>([]);
  const frameStop = signal(0);
  const selectedFrame = signal(-1);
  const rowEpoch = frameStop.handle();
  const rowSelection = selectedFrame.handle();
  const frameRequests = requests.sender();
  const locals = signal<string[]>([]);
  const localsOpen = signal(true);
  const stackOpen = signal(true);
  const unresolved = signal<UnresolvedBreakpoint[]>([]);
  const rowUnresolved = unresolved.handle();
  const listRunning = running.handle();
  const breakpointWorkspace = navigation.workspace.handle();
  const breakpointRecovery = navigation.recovery.handle();
  const breakpointOutput = output.handle();
  const listFont = font.handle();
  const toggleRecovery = navigation.recovery.handle();
  const launchWorkspace = workspace.handle();
  const listHandle = listed.handle();
  const toggleList = listed.handle();
  return (
    <View className="flex-col size-full min-h-0">
      <View className="shrink-0 flex-row items-center w-full min-w-0 border-b border-border">
        <View hidden={running.get()} className="grow flex-row items-center min-w-0">
          <Button
            className={'grow min-w-0 justify-start rounded-none px-2 min-h-[' + (font.get() + 12).toString() + 'px]'}
            aria-label={translate(lastTarget.get() === '' ? 'Launch executable…' : 'Run last executable')}
            tooltipCommand="debug-continue"
            tooltip={lastTarget.get() === '' ? translate('Launch executable…') : lastTarget.get()}
            icon="play"
            variant="ghost"
            size="xsmall"
            disabled={running.get()}
            onPress={() => {
              if (lastTarget.get() === '') chooseDebugTarget(chosen);
              else chosen.set(lastTarget.get());
            }}
          >
            <Text className={'text-[' + font.get().toString() + 'px]'}>
              {translate(lastTarget.get() === '' ? 'Launch executable…' : 'Run last executable')}
            </Text>
          </Button>
          <View hidden={lastTarget.get() === ''}>
            <Button
              icon="folder-open"
              size="xsmall"
              variant="ghost"
              tooltip={message('Choose another executable…')}
              aria-label={accessibleName('Choose another executable…')}
              onPress={() => {
                chooseDebugTarget(chosen);
              }}
            />
          </View>
          <Button
            icon="circle-pause"
            tooltip={message('Stop on entry')}
            aria-label={accessibleName('Stop on entry')}
            selected={stopOnEntry.get()}
            disabled={running.get()}
            variant="ghost"
            size="xsmall"
            onPress={() => {
              stopOnEntry.set(!stopOnEntry.get());
            }}
          />
          <Button
            variant="ghost"
            size="xsmall"
            icon="circle-dot"
            disabled={running.get() || workspace.get().active === ''}
            tooltipCommand="toggle-source-breakpoint"
            tooltip={message('Toggle source breakpoint at cursor')}
            aria-label={accessibleName('Toggle source breakpoint at cursor')}
            onPress={() => {
              toggleCursorBreakpoint(selectedWorkspace, toggleRecovery, toggleList, running.get());
            }}
          />
        </View>
        <View hidden={!stopped.get()} className="flex-row items-center min-w-0">
          {DebugControl(
            requests,
            stopped,
            output,
            'continue',
            'Continue',
            'play'
          )}
          {DebugControl(
            requests,
            stopped,
            output,
            'stepOver',
            'Step over',
            'corner-down-right'
          )}
          {DebugControl(
            requests,
            stopped,
            output,
            'stepIn',
            'Step into',
            'arrow-down'
          )}
          {DebugControl(
            requests,
            stopped,
            output,
            'stepOut',
            'Step out',
            'arrow-up'
          )}
          {DebugControl(
            requests,
            stopped,
            output,
            'next',
            'Step instruction',
            'step-forward'
          )}
        </View>
        <View hidden={!running.get() || stopped.get()} className="flex-row items-center min-w-0">
          {DebugControl(
            requests,
            stopped,
            output,
            'pause',
            'Pause debugging',
            'pause'
          )}
        </View>
        <View hidden={!running.get()} className="flex-row items-center min-w-0">
          {DebugControl(
            requests,
            stopped,
            output,
            'stop',
            'Stop debugging',
            'square'
          )}
        </View>
      </View>
      <View className="flex-col grow w-full min-w-0 min-h-0 overflow-y-auto">
        {...listed.get().map((point: ReadOnly<SourceBreakpoint>) => BreakpointRow(
          clone(point),
          rowUnresolved.handle(),
          listHandle.handle(),
          root,
          listRunning.handle(),
          listFont.handle(),
          {
            workspace: breakpointWorkspace.handle(), recovery: breakpointRecovery.handle(), allowedRoot: clone(root), allowNavigation: true, focusEditor: true
          },
          breakpointOutput.handle()
        ))}
        <View hidden={running.get()} className="w-full min-w-0">
          <InputGroup size="small" className="w-full h-8 shrink-0 min-w-0 rounded-none border border-transparent">
            <Input
              className={'px-2 text-[' + font.get().toString() + 'px]'}
              value={breakpoint.get()}
              placeholder={message('Function breakpoint')}
              aria-label={accessibleName('Function breakpoint')}
              onChange={(event) => {
                breakpoint.set(clone(event.target.value));
              }}
            />
          </InputGroup>
        </View>
        <View hidden={running.get() && !stopped.get() && watch.get() === ''} className="flex-col w-full min-w-0">
          <InputGroup size="small" className="w-full h-8 shrink-0 min-w-0 rounded-none border border-transparent">
            <Input
              className={'px-2 text-[' + font.get().toString() + 'px]'}
              value={watch.get()}
              disabled={running.get() && !stopped.get()}
              placeholder={translate(stopped.get() ? 'Press Enter to evaluate the watch' : 'Watch expression')}
              aria-label={accessibleName('Watch expression')}
              onChange={(event) => {
                watch.set(clone(event.target.value));
                watchValue.set('');
              }}
              onSubmit={(_secondary) => {
                if (!stopped.get()) return;

                match (watchSender.trySend({
                  action: 'setWatch', stopId: frameStop.get(), watch: watch.get()
                })) {
                  Ok(_) => {
                    stopped.set(false);
                  },
                  Err(error) => {
                    output.set(output.get() + error.code);
                  }
                }
              }}
            />
          </InputGroup>
          <View hidden={!stopped.get() || watchValue.get() === ''}>
            <Input appearance={false} bordered={false} size="small" height={font.get() + 12} readOnly={true} value={watchValue.get()} aria-label={accessibleName('Watch value')} className={'w-full min-w-0 px-2 font-mono text-[' + font.get().toString() + 'px]'} />
          </View>
        </View>
        <View hidden={!stopped.get() || locals.get().length === 0} className="w-full flex-col min-w-0">
          {InspectionHeading('Local variables', localsOpen.handle(), font.handle())}
          <View hidden={!localsOpen.get()} className="w-full flex-col min-w-0">
            {...locals.get().map((value: ReadOnly<string>) => (
              <Input appearance={false} bordered={false} size="small" height={font.get() + 12} readOnly={true} value={clone(value)} aria-label={accessibleName('Local variable')} className={'w-full min-w-0 px-2 font-mono text-[' + font.get().toString() + 'px]'} />
            ))}
          </View>
        </View>
        <View hidden={!stopped.get() || frames.get().length === 0} className="w-full flex-col min-w-0">
          {InspectionHeading('Call stack', stackOpen.handle(), font.handle())}
          <View hidden={!stackOpen.get()} className="w-full flex-col min-w-0">
            {...frames.get().map((frame: ReadOnly<SourceFrame>) => StackFrameRow(
              clone(frame),
              frameRoot,
              rowEpoch.get(),
              rowSelection.handle(),
              frameRequests.share(),
              frameStopped.handle(),
              frameOutput.handle(),
              frameFont.handle()
            ))}
          </View>
        </View>
      </View>
      <Ticker interval={50} running={running.get() || chosen.get() !== ''} onTick={(_frame) => {
        if (chosen.get() !== '') {
          const path = chosen.get();
          chosen.set('');
          match (sender.trySend({action: 'launch', program: clone(path), stopOnEntry: stopOnEntry.get(), functionBreakpoint: breakpoint.get(), sourceBreakpoints: clone(launchWorkspace.get().sourceBreakpoints), watch: watch.get()})) {
            Ok(_) => {
              unresolved.set([]);
              lastTarget.set(clone(path));
              output.set(path + '\n');
              running.set(true);
              panel.set('debug');
            },
            Err(error) => { output.set(clone(error.code)); return; }
          }
        }
        match (replies.tryReceive()) {
          Ok(reply) => {
            if (reply.finished) unresolved.set([]);
            else if (reply.verifiedBreakpoint !== undefined) {
              const point = clone(reply.verifiedBreakpoint);
              unresolved.set(unresolved.get().filter((entry) => entry.path !== point.path || entry.line !== point.line));
            } else if (reply.unresolvedBreakpoint !== undefined) {
              unresolved.set([...unresolved.get(), clone(reply.unresolvedBreakpoint)]);
            }
            if (reply.stopped === true) {
              frameStop.set(reply.stopId ?? 0);
              selectedFrame.set(reply.selectedFrame ?? -1);
              frames.set(clone(reply.frames) ?? []);
              locals.set(clone(reply.locals) ?? []);
              watchValue.set(reply.watchExpression === watch.get().trim() ? clone(reply.watchValue ?? '') : '');
            }
            const location = reply.location;
            if (location !== undefined) openDebugLocation(navigation, location.path, location.line, location.column, output);
            output.set(output.get() + reply.text);
            running.set(!reply.finished);
            if (reply.stopped === true || reply.finished) stopped.set(reply.stopped ?? false);
          },
          Err(error) => { if (error.code !== 'empty') { output.set(output.get() + error.code); running.set(false); } }
        }
      }} />
    </View>
  );
}

export function DebugConsole(output: Signal<string>, font: Signal<int>): Node {
  const style = computed<string>(() => 'grow min-h-0 font-mono text-[' + font.get().toString() + 'px]');
  return (
    <View className="flex-col size-full min-h-0">
      <CodeEditor value={output.get()} readOnly={true} appearance={false} bordered={false} className={style.get()} accessibilityLabel={message('Debug Console')} />
    </View>
  );
}
