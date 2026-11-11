import { accessibleName, message } from './localization.tsr';
import { View, Text, Button, Tabs, Tab, Node, Signal, t } from 'std:ui';

export function Panels(panel: Signal<string>): Node {
  return <View className="flex-col size-full min-h-0 bg-muted">
    <View className="flex-row items-center shrink-0 border-b border-border">
      <Tabs value={panel.get()} variant="underline" size="small" className="grow min-w-0" onChange={(value) => { panel.set(clone(value)); }}>
        <Tab value="terminal" label={message("Terminal")} /><Tab value="problems" label={message("Problems")} /><Tab value="output" label={message("Output")} /><Tab value="debug" label={message("Debug Console")} /><Tab value="git" label={message("Source Control")} /><Tab value="extensions" label={message("Extensions")} />
      </Tabs>
      <Button icon="x" variant="ghost" size="xsmall" tooltip={message("Close panel")} aria-label={accessibleName("Close panel")} onPress={() => { panel.set(''); }} />
    </View>
    <View className="p-3 grow min-h-0">
      <View hidden={panel.get() !== 'terminal'}><Text>{t("Terminal integration is planned. No shell session is running.")}</Text></View>
      <View hidden={panel.get() !== 'problems'}><Text>{t("Language-service diagnostics are not connected yet.")}</Text></View>
      <View hidden={panel.get() !== 'output'}><Text>{t("Build and task output will appear here when task execution is connected.")}</Text></View>
      <View hidden={panel.get() !== 'debug'}><Text>{t("Debugger integration is planned. No debug session is running.")}</Text></View>
      <View hidden={panel.get() !== 'git'}><Text>{t("Repository status, diffs and staging are planned.")}</Text></View>
      <View hidden={panel.get() !== 'extensions'}><Text>{t("Extension management is planned.")}</Text></View>
    </View>
  </View>;
}

export function ChatPanel(open: Signal<boolean>): Node {
  return <View className="flex-col size-full min-h-0">
    <View className="flex-row items-center px-3 py-1 border-b border-border">
      <Text className="grow">{t("AI Chat")}</Text>
      <Button icon="x" variant="ghost" size="xsmall" tooltip={message("Close AI Chat")} aria-label={accessibleName("Close AI Chat")} onPress={() => { open.set(false); }} />
    </View>
    <View className="p-3"><Text>{t("Agent chat and proposed-change review are planned. No agent is running.")}</Text></View>
  </View>;
}
