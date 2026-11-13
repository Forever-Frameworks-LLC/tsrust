import { Channel } from 'std:futures';
import {
  Button,
  CodeEditor,
  Node,
  Signal,
  Ticker,
  View,
  computed,
  pickFilePaths,
  signal,
  t
} from 'std:ui';
import { message } from '../../lib/localization.tsr';
import { DebugCommand, DebugOutput } from '../../services/debug/session.tsr';

export function DebugConsole(requests: ReadOnly<Channel<DebugCommand>>, replies: Channel<DebugOutput>, font: Signal<int>): Node {
  const sender = requests.sender();
  const controls = requests.sender();
  const steps = requests.sender();
  const stopOnEntry = signal(false);
  const stopped = signal(false);
  const chosen = signal('');
  const running = signal(false);
  const output = signal('');
  const style = computed<string>(() => 'grow min-h-0 font-mono text-[' + font.get().toString() + 'px]');
  return (
    <View className="flex-col size-full min-h-0">
      <View className="flex-row items-center shrink-0">
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
          {t('Launch executable…')}
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
          {t('Stop on entry')}
        </Button>
        <View hidden={!stopped.get()} className="flex-row">
          <Button
            variant="ghost"
            size="small"
            onPress={() => {
              match (controls.trySend({
                action: 'continue'
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
            {t('Continue')}
          </Button>
          <Button
            variant="ghost"
            size="small"
            onPress={() => {
              match (steps.trySend({
                action: 'next'
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
            {t('Step instruction')}
          </Button>
        </View>
      </View>
      <CodeEditor value={output.get()} readOnly={true} appearance={false} bordered={false} className={style.get()} accessibilityLabel={message('Debug Console')} />
      <Ticker interval={50} running={running.get() || chosen.get() !== ''} onTick={(_frame) => {
        if (chosen.get() !== '') {
          const path = chosen.get();
          chosen.set('');
          match (sender.trySend({action: 'launch', program: clone(path), stopOnEntry: stopOnEntry.get()})) {
            Ok(_) => { output.set(path + '\n'); running.set(true); },
            Err(error) => { output.set(clone(error.code)); return; }
          }
        }
        match (replies.tryReceive()) {
          Ok(reply) => { output.set(output.get() + reply.text); running.set(!reply.finished); if (reply.stopped === true || reply.finished) stopped.set(reply.stopped ?? false); },
          Err(error) => { if (error.code !== 'empty') { output.set(output.get() + error.code); running.set(false); } }
        }
      }} />
    </View>
  );
}
