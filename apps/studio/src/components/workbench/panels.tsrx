import {
  View,
  Text,
  Button,
  Tabs,
  Tab,
  TabsSuffix,
  Node,
  Signal,
  computed,
  t
} from 'std:ui';

import { accessibleName, message } from '../../lib/localization.tsr';

export function Panels(
  panel: Signal<string>,
  output: Node,
  terminal: Node,
  terminalActions: Node,
  problems: Node,
  problemsActions: Node,
  font: Signal<int>
): Node {
  const fontClass = computed<string>(() => 'flex-col size-full min-h-0 bg-muted text-[' + font.get().toString() + 'px]');
  const tabsClass = computed<string>(() => 'w-full min-w-0 text-[' + font.get().toString() + 'px]');
  return (
    <View className={fontClass.get()}>
      <View className="flex-row items-center shrink-0">
        <Tabs
          value={panel.get()}
          variant="underline"
          size="small"
          className={tabsClass.get()}
          onChange={(value) => {
            panel.set(clone(value));
          }}
        >
          <Tab value="terminal" label={message("Terminal")} />
          <Tab value="problems" label={message("Problems")} />
          <Tab value="output" label={message("Output")} />
          <Tab value="debug" label={message("Debug Console")} />
          <Tab value="git" label={message("Source Control")} />
          <Tab value="extensions" label={message("Extensions")} />
          <TabsSuffix>
            <View className="flex-row items-center pr-3">
              <View hidden={panel.get() !== 'terminal'}>
                {terminalActions}
              </View>
              <View hidden={panel.get() !== 'problems'}>
                {problemsActions}
              </View>
              <Button
                icon="x"
                variant="ghost"
                size="xsmall"
                tooltip={message("Close panel")}
                aria-label={accessibleName("Close panel")}
                onPress={() => {
                  panel.set('');
                }}
              />
            </View>
          </TabsSuffix>
        </Tabs>
      </View>
      <View className="grow min-h-0">
        <View hidden={panel.get() !== 'terminal'} className="size-full min-h-0">
          {terminal}
        </View>
        <View hidden={panel.get() !== 'problems'} className="size-full min-h-0">
          {problems}
        </View>
        <View hidden={panel.get() !== 'output'} className="size-full min-h-0">
          {output}
        </View>
        <View hidden={panel.get() !== 'debug'}>
          <Text>
            {t("Debugger integration is planned. No debug session is running.")}
          </Text>
        </View>
        <View hidden={panel.get() !== 'git'}>
          <Text>
            {t("Repository status, diffs and staging are planned.")}
          </Text>
        </View>
        <View hidden={panel.get() !== 'extensions'}>
          <Text>
            {t("Extension management is planned.")}
          </Text>
        </View>
      </View>
    </View>
  );
}

export function ChatPanel(open: Signal<boolean>): Node {
  return (
    <View className="flex-col size-full min-h-0">
      <View className="flex-row items-center px-3 py-1 border-b border-border">
        <Text className="grow">
          {t("AI Chat")}
        </Text>
        <Button
          icon="x"
          variant="ghost"
          size="xsmall"
          tooltip={message("Close AI Chat")}
          aria-label={accessibleName("Close AI Chat")}
          onPress={() => {
            open.set(false);
          }}
        />
      </View>
      <View className="p-3">
        <Text>
          {t("Agent chat and proposed-change review are planned. No agent is running.")}
        </Text>
      </View>
    </View>
  );
}
