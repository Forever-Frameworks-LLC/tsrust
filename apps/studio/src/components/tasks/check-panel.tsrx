import { View, Text, Button, CodeEditor, Ticker, Shortcut, Node, Signal, signal, computed, t } from 'std:ui';
import { Channel, Sender } from 'std:futures';
import { CheckResult } from '../../services/project-check.tsr';
import { message, accessibleName } from '../../lib/localization.tsr';

export function CheckPanel(panel: Signal<string>, requests: Sender<string>, shortcutRequests: Sender<string>, cancellations: Sender<string>, results: Channel<CheckResult>, font: Signal<int>): Node {
  const running = signal(false);
  const output = signal('');
  const editorClass = computed<string>(() => 'size-full min-h-0 font-mono text-[' + font.get().toString() + 'px]');
  return <View className="flex-col size-full min-h-0">
    <View className="flex-row items-center shrink-0 border-b border-border px-2 h-8 gap-2">
      <Button icon="play" variant="ghost" size="xsmall" disabled={running.get()} tooltip={message("Check project (Cmd+Shift+B)")} aria-label={accessibleName("Check project")} onPress={() => {
        match (requests.trySend('check')) {
          Ok(_) => { running.set(true); output.set(''); },
          Err(error) => { output.set(clone(error.code)); }
        }
      }} />
      <Button icon="square" variant="ghost" size="xsmall" disabled={!running.get()} tooltip={message("Cancel project check")} aria-label={accessibleName("Cancel project check")} onPress={() => {
        match (cancellations.trySend('cancel')) { Ok(_) => {}, Err(error) => { output.set(clone(error.code)); } }
      }} />
      <Text className="text-xs">{running.get() ? t("Checking saved project files…") : t("Project check")}</Text>
    </View>
    <CodeEditor value={output.get()} readOnly={true} appearance={false} bordered={false} className={editorClass.get()} accessibilityLabel={message("Project check output")} />
    <Ticker interval={100} running={running.get()} onTick={(_frame) => {
      match (results.tryReceive()) {
        Ok(result) => { output.set(clone(result.output)); running.set(false); },
        Err(error) => { if (error.code !== 'empty') { output.set(clone(error.code)); running.set(false); } }
      }
    }} />
    <Shortcut command="check-project" keys="secondary-shift-b" onTrigger={() => {
      panel.set('output');
      if (running.get()) return;
      match (shortcutRequests.trySend('check')) {
        Ok(_) => { running.set(true); output.set(''); },
        Err(error) => { output.set(clone(error.code)); }
      }
    }} />
  </View>;
}
