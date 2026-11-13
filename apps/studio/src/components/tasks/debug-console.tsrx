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
import { DebugOutput } from '../../services/debug/session.tsr';

export function DebugConsole(requests: ReadOnly<Channel<string>>, replies: Channel<DebugOutput>, font: Signal<int>): Node {
  const sender = requests.sender();
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
      </View>
      <CodeEditor value={output.get()} readOnly={true} appearance={false} bordered={false} className={style.get()} accessibilityLabel={message('Debug Console')} />
      <Ticker interval={50} running={running.get() || chosen.get() !== ''} onTick={(_frame) => {
        if (chosen.get() !== '') {
          const path = chosen.get();
          chosen.set('');
          match (sender.trySend(clone(path))) {
            Ok(_) => { output.set(path + '\n'); running.set(true); },
            Err(error) => { output.set(clone(error.code)); return; }
          }
        }
        match (replies.tryReceive()) {
          Ok(reply) => { output.set(output.get() + reply.text); running.set(!reply.finished); },
          Err(error) => { if (error.code !== 'empty') { output.set(output.get() + error.code); running.set(false); } }
        }
      }} />
    </View>
  );
}
