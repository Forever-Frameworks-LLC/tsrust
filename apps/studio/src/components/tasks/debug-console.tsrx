import { Channel } from 'std:futures';
import {
  Button,
  CodeEditor,
  Input,
  Text,
  Node,
  Signal,
  Ticker,
  View,
  computed,
  pickFilePaths,
  signal,
  t
} from 'std:ui';
import { message, accessibleName } from '../../lib/localization.tsr';
import { DebugCommand, DebugOutput } from '../../services/debug/session.tsr';

import { SearchActionContext } from '../../lib/explorer/search-actions.tsr';
import { openLanguageLocation } from '../../lib/language/navigation.tsr';
import { DiagnosticPosition, documentUri } from '../../lib/language/diagnostics.tsr';

function DebugControl(
  requests: ReadOnly<Channel<DebugCommand>>,
  stopped: Signal<boolean>,
  output: Signal<string>,
  action: string,
  label: string,
  font: Signal<int>
): Node {
  const sender = requests.sender();
  return (
    <Button
      variant="ghost"
      size="small"
      onPress={() => {
        match (sender.trySend({
          action: clone(action)
        })) {
          Ok(_) => {
            stopped.set(false);
          },
          Err(error) => {
            output.set(output.get() + error.code);
          }
        }
      }}
    >
      <Text className={'text-[' + font.get().toString() + 'px]'}>
        {t(label)}
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
  navigation: SearchActionContext
): Node {
  const sender = requests.sender();
  const stopOnEntry = signal(false);
  const breakpoint = signal('');
  const watch = signal('');
  const watchValue = signal('');
  const stopped = signal(false);
  const chosen = signal('');
  const running = signal(false);
  return (
    <View className="flex-col size-full min-h-0">
      <View className="flex-col w-full min-w-0 items-start gap-1 px-2 py-1">
        <Button
          icon="play"
          variant="ghost"
          size="small"
          disabled={running.get()}
          onPress={() => {
            const target = chosen.handle();
            pickFilePaths((paths) => {
              if (paths.length > 0) target.set(clone(paths[0]));
            }, {
              multiple: false
            });
          }}
        >
          <Text className={'text-[' + font.get().toString() + 'px]'}>
            {t('Launch executable…')}
          </Text>
        </Button>
        <Button
          selected={stopOnEntry.get()}
          disabled={running.get()}
          variant="ghost"
          size="small"
          onPress={() => {
            stopOnEntry.set(!stopOnEntry.get());
          }}
        >
          <Text className={'text-[' + font.get().toString() + 'px]'}>
            {t('Stop on entry')}
          </Text>
        </Button>
        <View hidden={running.get()} className="w-full min-w-0">
          <Input
            className={'px-2 text-[' + font.get().toString() + 'px]'}
            value={breakpoint.get()}
            placeholder={message('Function breakpoint')}
            aria-label={accessibleName('Function breakpoint')}
            onChange={(event) => {
              breakpoint.set(clone(event.target.value));
            }}
          />
        </View>
        <View hidden={running.get() && watch.get() === ''} className="flex-col w-full min-w-0">
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
          <View hidden={!stopped.get() || watchValue.get() === ''}>
            <Input readOnly={true} value={watchValue.get()} aria-label={accessibleName('Watch value')} className={'font-mono text-[' + font.get().toString() + 'px]'} />
          </View>
        </View>
        <View hidden={!stopped.get()} className="flex-col items-start">
          {DebugControl(
            requests,
            stopped,
            output,
            'continue',
            'Continue',
            font
          )}
          {DebugControl(
            requests,
            stopped,
            output,
            'next',
            'Step instruction',
            font
          )}
          {DebugControl(
            requests,
            stopped,
            output,
            'stop',
            'Stop debugging',
            font
          )}
        </View>
      </View>
      <Ticker interval={50} running={running.get() || chosen.get() !== ''} onTick={(_frame) => {
        if (chosen.get() !== '') {
          const path = chosen.get();
          chosen.set('');
          match (sender.trySend({action: 'launch', program: clone(path), stopOnEntry: stopOnEntry.get(), functionBreakpoint: breakpoint.get(), watch: watch.get()})) {
            Ok(_) => { output.set(path + '\n'); running.set(true); panel.set('debug'); },
            Err(error) => { output.set(clone(error.code)); return; }
          }
        }
        match (replies.tryReceive()) {
          Ok(reply) => {
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
            output.set(output.get() + reply.text); running.set(!reply.finished); if (reply.stopped === true) watchValue.set(clone(reply.watchValue ?? '')); if (reply.stopped === true || reply.finished) stopped.set(reply.stopped ?? false); },
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
