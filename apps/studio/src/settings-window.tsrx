import { View, Text, Button, CodeEditor, Select, Node, Signal, signal, computed, openWindow, useTheme } from 'std:ui';
import { configPath, loadConfig, saveConfig, parseConfig, defaultConfig, themes, StudioConfig } from './settings.tsr';

function SettingsEditor(path: string, font: Signal<int>, settings: Signal<StudioConfig>): Node {
  let initial = defaultConfig();
  const message = signal('');
  match (loadConfig(path)) {
    Ok(config) => {
      initial = config;
    },
    Err(error) => {
      message.set(error + ' The existing settings file has been preserved.');
    }
  }

  const text = signal(JSON.stringify(initial));
  const selectedTheme = computed<string>(() => {
    match (parseConfig(text.get())) {
      Ok(config) => {
        return clone(config.theme);
      },
      Err(_) => {
        return '';
      }
    }
  });
  const savePath = clone(path);
  const reloadPath = clone(path);
  return <View className="flex-col size-full bg-background text-foreground">
    <View className="flex-row items-center gap-2 p-3 border-b border-border">
      <Text className="grow truncate">{path}</Text>
      <Button icon="save" size="small" onPress={() => {
        match (saveConfig(savePath, text.get())) {
          Ok(config) => { font.set(config.fontSize); useTheme(clone(config.theme)); settings.set(config); message.set('Settings saved.'); },
          Err(error) => { message.set(error); }
        }
      }}>Save</Button>
      <Button icon="rotate-ccw" size="small" variant="ghost" onPress={() => {
        match (loadConfig(reloadPath)) { Ok(config) => { text.set(JSON.stringify(config)); message.set(''); }, Err(error) => { message.set(error); } }
      }}>Reload</Button>
    </View>
    <View className="flex-row items-center gap-3 px-3 py-2 border-b border-border">
      <Text>Theme</Text>
      <Select items={themes()} value={selectedTheme.get()} searchable={true} accessibilityLabel="Color theme" className="w-64" onChange={(name) => {
        match (parseConfig(text.get())) {
          Ok(config) => { config.theme = clone(name); text.set(JSON.stringify(config)); useTheme(clone(name)); message.set('Theme preview. Save to keep this configuration.'); },
          Err(error) => { message.set(error); }
        }
      }} />
    </View>
    <CodeEditor language="json" value={text.get()} onChange={(value) => { text.set(clone(value)); }} className="grow min-h-0 text-[14px]" appearance={false} bordered={false} accessibilityLabel="Studio configuration" />
    <View className="px-3 py-2 shrink-0"><Text>{message.get()}</Text></View>
  </View>;
}

export function showSettings(font: Signal<int>, settings: Signal<StudioConfig>): void {
  match (configPath()) {
    Ok(path) => {
      openWindow(SettingsEditor(path, font, settings), 'Tsrust Studio Settings', {
        width: 820, height: 640, minWidth: 600, minHeight: 400
      });
    },
    Err(error) => {
      openWindow(<View className="p-4"><Text>{error}</Text></View>, 'Studio Settings Error', {
        width: 600, height: 180
      });
    }
  }
}
