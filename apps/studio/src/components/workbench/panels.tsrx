import {
  View,
  Button,
  Tabs,
  Tab,
  TabsSuffix,
  Node,
  Signal,
  computed
} from 'std:ui';

import { accessibleName, message } from '../../lib/localization.tsr';

import { StudioConfig } from '../../lib/settings/settings.tsr';
import { availableModelPaths } from '../../services/ai/model-extensions.tsr';

export function selectPanel(panel: Signal<string>, settings: Signal<StudioConfig>, modelPaths: Signal<string[]>, value: ReadOnly<string>): void {
  if (value === 'extensions') modelPaths.set(availableModelPaths(settings.get()));
  panel.set(clone(value));
}

export function Panels(
  panel: Signal<string>,
  settings: Signal<StudioConfig>,
  modelPaths: Signal<string[]>,
  output: Node,
  terminal: Node,
  terminalActions: Node,
  problems: Node,
  problemsActions: Node,
  references: Node,
  debug: Node,
  extensions: Node,
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
            selectPanel(panel, settings, modelPaths, value);
          }}
        >
          <Tab value="terminal" label={message("Terminal")} />
          <Tab value="problems" label={message("Problems")} />
          <Tab value="references" label={message("References")} />
          <Tab value="output" label={message("Output")} />
          <Tab value="debug" label={message("Debug Console")} />
          <Tab value="extensions" label={message("Extensions")} />
          <TabsSuffix>
            <View className="flex-row items-center pr-1.5">
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
        <View hidden={panel.get() !== 'references'} className="size-full min-h-0">
          {references}
        </View>
        <View hidden={panel.get() !== 'output'} className="size-full min-h-0">
          {output}
        </View>
        <View hidden={panel.get() !== 'debug'} className="size-full min-h-0">
          {debug}
        </View>
        <View hidden={panel.get() !== 'extensions'} className="size-full min-h-0">
          {extensions}
        </View>
      </View>
    </View>
  );
}
