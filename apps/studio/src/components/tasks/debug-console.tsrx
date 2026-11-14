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
  t
} from 'std:ui';

import { SearchActionContext } from '../../lib/explorer/search-actions.tsr';
import { DiagnosticPosition, documentUri } from '../../lib/language/diagnostics.tsr';
import { openLanguageLocation } from '../../lib/language/navigation.tsr';
import { message, accessibleName } from '../../lib/localization.tsr';
import { DebugCommand, DebugOutput, SourceFrame, SourceBreakpoint, toggleSourceBreakpoint } from '../../services/debug/session.tsr';

export function toggleCursorBreakpoint(active: ReadOnly<string>, lines: Signal<SourceBreakpoint[]>, running: boolean): void {
  if (running || active === '') return;
  const path = clone(active);
  readInput(clone(path), (field) => {
    if (!field.found) return;
    const caret = field.selectionReversed === true ? field.selectionStart : field.selectionEnd;

    if (caret === undefined || field.value === undefined) return;
    const points = lines.get();
    toggleSourceBreakpoint(points, clone(path), field.value.slice(0, caret).split('\n').length);
    lines.set(points);
  });
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

function BreakpointRow(target: SourceBreakpoint, root: ReadOnly<string>, lines: Signal<SourceBreakpoint[]>, running: Signal<boolean>, font: Signal<int>): Node {
  const label = target.path.slice(root.length + 1) + ':' + target.line.toString();
  return (
    <Button
      className={'w-full min-w-0 justify-start px-2 py-[6px] rounded-none border-b border-border hover:bg-accent text-[' + font.get().toString() + 'px]'}
      variant="ghost"
      size="small"
      disabled={running.get()}
      tooltip={message('Remove breakpoint')}
      onPress={() => {
        const points = lines.get();
        const index = points.findIndex((item) => item.path === target.path && item.line === target.line);

        if (index >= 0) points.splice(index, 1);
        lines.set(points);
      }}
    >
      <Text className={'truncate text-[' + font.get().toString() + 'px]'}>
        {label}
      </Text>
    </Button>
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
  const name = frame.location.functionName ?? '';
  const path = frame.location.path;
  const label = '#' + frame.depth.toString() + ' ' + (name === '' ? '' : name + '  ') + (path.startsWith(root + '/') ? path.slice(root.length + 1) : clone(path)) + ':' + frame.location.line.toString();
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

export function DebugSidebar(
  requests: ReadOnly<Channel<DebugCommand>>,
  replies: Channel<DebugOutput>,
  font: Signal<int>,
  output: Signal<string>,
  panel: Signal<string>,
  navigation: SearchActionContext,
  lines: Signal<SourceBreakpoint[]>,
  running: Signal<boolean>,
  stopped: Signal<boolean>,
  chosen: Signal<string>
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
  const watch = signal('');
  const watchValue = signal('');
  const frames = signal<SourceFrame[]>([]);
  const frameStop = signal(0);
  const selectedFrame = signal(-1);
  const rowEpoch = frameStop.handle();
  const rowSelection = selectedFrame.handle();
  const frameRequests = requests.sender();
  const locals = signal<string[]>([]);
  const listRunning = running.handle();
  const listFont = font.handle();
  const listLines = lines.handle();
  return (
    <View className="flex-col size-full min-h-0">
      <View className="shrink-0 flex-row items-center w-full min-w-0 border-b border-border">
        <View hidden={running.get()} className="grow flex-row items-center min-w-0">
          <Button
            className="grow min-w-0 justify-start rounded-none px-2"
            aria-label={accessibleName('Launch executable…')}
            tooltipCommand="debug-continue"
            tooltip={message('Launch executable…')}
            icon="play"
            variant="ghost"
            size="xsmall"
            disabled={running.get()}
            onPress={() => {
              chooseDebugTarget(chosen);
            }}
          >
            <Text className={'text-[' + font.get().toString() + 'px]'}>
              {t('Launch executable…')}
            </Text>
          </Button>
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
              toggleCursorBreakpoint(selectedWorkspace.get().active, lines, running.get());
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
        {...lines.get().map((point: ReadOnly<SourceBreakpoint>) => BreakpointRow(clone(point), root, listLines.handle(), listRunning.handle(), listFont.handle()))}
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
        <View hidden={running.get() && watch.get() === ''} className="flex-col w-full min-w-0">
          <InputGroup size="small" className="w-full h-8 shrink-0 min-w-0 rounded-none border border-transparent">
            <Input
              className={'px-2 text-[' + font.get().toString() + 'px]'}
              value={watch.get()}
              disabled={running.get()}
              placeholder={message('Watch expression')}
              aria-label={accessibleName('Watch expression')}
              onChange={(event) => {
                watch.set(clone(event.target.value));
              }}
            />
          </InputGroup>
          <View hidden={!stopped.get() || watchValue.get() === ''}>
            <Input readOnly={true} value={watchValue.get()} aria-label={accessibleName('Watch value')} className={'font-mono text-[' + font.get().toString() + 'px]'} />
          </View>
        </View>
        <View hidden={!stopped.get() || frames.get().length === 0} className="w-full flex-col min-w-0">
          <Text className={'px-2 py-1 text-muted-foreground text-[' + font.get().toString() + 'px]'}>
            {t('Call stack')}
          </Text>
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
        <View hidden={!stopped.get() || locals.get().length === 0} className="w-full flex-col min-w-0">
          <Text className={'px-2 py-1 text-muted-foreground text-[' + font.get().toString() + 'px]'}>
            {t('Local variables')}
          </Text>
          {...locals.get().map((value: ReadOnly<string>) => (
            <Input readOnly={true} value={clone(value)} aria-label={accessibleName('Local variable')} className={'px-2 font-mono text-[' + font.get().toString() + 'px]'} />
          ))}
        </View>
      </View>
      <Ticker interval={50} running={running.get() || chosen.get() !== ''} onTick={(_frame) => {
        if (chosen.get() !== '') {
          const path = chosen.get();
          chosen.set('');
          match (sender.trySend({action: 'launch', program: clone(path), stopOnEntry: stopOnEntry.get(), functionBreakpoint: breakpoint.get(), sourceBreakpoints: lines.get(), watch: watch.get()})) {
            Ok(_) => { output.set(path + '\n'); running.set(true); panel.set('debug'); },
            Err(error) => { output.set(clone(error.code)); return; }
          }
        }
        match (replies.tryReceive()) {
          Ok(reply) => {
            if (reply.stopped === true) {
              frameStop.set(reply.stopId ?? 0);
              selectedFrame.set(reply.selectedFrame ?? -1);
              frames.set(clone(reply.frames) ?? []);
            }
            const location = reply.location;
            if (location !== undefined) {
              const point: DiagnosticPosition = {line: location.line - 1, character: location.column - 1};
              match (openLanguageLocation(navigation, {
                root: clone(navigation.allowedRoot ?? ''), uri: documentUri(location.path),
                selection: {start: clone(point), end: point}
              })) {
                Ok(_) => {},
                Err(error) => { output.set(output.get() + error + '\n'); }
              }
            }
            output.set(output.get() + reply.text); running.set(!reply.finished); if (reply.stopped === true) locals.set(clone(reply.locals) ?? []); if (reply.stopped === true) watchValue.set(clone(reply.watchValue ?? '')); if (reply.stopped === true || reply.finished) stopped.set(reply.stopped ?? false); },
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
