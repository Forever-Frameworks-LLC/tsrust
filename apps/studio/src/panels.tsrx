import { View, Text, Button, Tabs, Tab, Node, Signal } from 'std:ui';

export function Panels(panel: Signal<string>): Node {
  return <View className="flex-col size-full min-h-0 bg-muted">
    <View className="flex-row items-center shrink-0 border-b border-border">
      <Tabs value={panel.get()} variant="underline" size="small" className="grow min-w-0" onChange={(value) => { panel.set(clone(value)); }}>
        <Tab value="terminal" label="Terminal" /><Tab value="problems" label="Problems" /><Tab value="output" label="Output" /><Tab value="debug" label="Debug Console" /><Tab value="git" label="Source Control" /><Tab value="extensions" label="Extensions" />
      </Tabs>
      <Button icon="x" variant="ghost" size="xsmall" tooltip="Close panel" aria-label="Close panel" onPress={() => { panel.set(''); }} />
    </View>
    <View className="p-3 grow min-h-0">
      <View hidden={panel.get() !== 'terminal'}><Text>Terminal integration is planned. No shell session is running.</Text></View>
      <View hidden={panel.get() !== 'problems'}><Text>Language-service diagnostics are not connected yet.</Text></View>
      <View hidden={panel.get() !== 'output'}><Text>Build and task output will appear here when task execution is connected.</Text></View>
      <View hidden={panel.get() !== 'debug'}><Text>Debugger integration is planned. No debug session is running.</Text></View>
      <View hidden={panel.get() !== 'git'}><Text>Repository status, diffs and staging are planned.</Text></View>
      <View hidden={panel.get() !== 'extensions'}><Text>Extension management is planned.</Text></View>
    </View>
  </View>;
}

export function ChatPanel(open: Signal<boolean>): Node {
  return <View className="flex-col size-full min-h-0">
    <View className="flex-row items-center px-3 py-1 border-b border-border">
      <Text className="grow">AI Chat</Text>
      <Button icon="x" variant="ghost" size="xsmall" tooltip="Close AI Chat" aria-label="Close AI Chat" onPress={() => { open.set(false); }} />
    </View>
    <View className="p-3"><Text>Agent chat and proposed-change review are planned. No agent is running.</Text></View>
  </View>;
}
