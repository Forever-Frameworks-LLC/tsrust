import { message, localizeError } from './localization.tsr';
import { View, Text, Button, CodeEditor, Select, SelectItem, Node, Signal, signal, computed, openWindow, useTheme, setLocale, setWindowTitle, translate, t } from 'std:ui';
import { configPath, loadConfig, saveConfig, parseConfig, defaultConfig, themes, StudioConfig } from './settings.tsr';

function applySettings(config: StudioConfig, font: Signal<int>, settings: Signal<StudioConfig>): void {
  setLocale(config.locale);
  font.set(config.fontSize);
  useTheme(clone(config.theme));
  settings.set(config);
  setWindowTitle(translate('Tsrust Studio Settings'));
}

function SettingsEditor(path: string, font: Signal<int>, settings: Signal<StudioConfig>): Node {
  let initial = defaultConfig();
  const notice = signal('');
  const preserved = signal(false);
  match (loadConfig(path)) {
    Ok(config) => {
      initial = config;
    },
    Err(error) => {
      notice.set(error);
      preserved.set(true);
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
  const selectedLocale = computed<string>(() => {
    match (parseConfig(text.get())) {
      Ok(config) => {
        return clone(config.locale);
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
          Ok(config) => { applySettings(config, font, settings); preserved.set(false); notice.set('Settings saved.'); },
          Err(error) => { notice.set(error); }
        }
      }}>{t("Save")}</Button>
      <Button icon="rotate-ccw" size="small" variant="ghost" onPress={() => {
        match (loadConfig(reloadPath)) { Ok(config) => { text.set(JSON.stringify(config)); applySettings(config, font, settings); preserved.set(false); notice.set(''); }, Err(error) => { notice.set(error); } }
      }}>{t("Reload")}</Button>
    </View>
    <View className="flex-row items-center gap-3 px-3 py-2 border-b border-border">
      <Text>{t("Theme")}</Text>
      <Select items={themes()} value={selectedTheme.get()} searchable={true} accessibilityLabel={message("Color theme")} className="w-64" onChange={(name) => {
        match (parseConfig(text.get())) {
          Ok(config) => { config.theme = clone(name); text.set(JSON.stringify(config)); useTheme(clone(name)); notice.set('Theme preview. Save to keep this configuration.'); },
          Err(error) => { notice.set(error); }
        }
      }} />
    </View>
    <View className="flex-row items-center gap-3 px-3 py-2 border-b border-border">
      <Text>{t("Language")}</Text>
      <Select value={selectedLocale.get()} accessibilityLabel={message("Language")} className="w-64" onChange={(value) => {
        match (parseConfig(text.get())) {
          Ok(config) => { config.locale = clone(value); text.set(JSON.stringify(config)); setLocale(value); setWindowTitle(translate('Tsrust Studio Settings')); notice.set('Language preview. Save to keep this configuration.'); },
          Err(error) => { notice.set(error); }
        }
      }}><SelectItem value="en" label="English" /><SelectItem value="pt-BR" label="Português (Brasil)" /></Select>
    </View>
    <CodeEditor language="json" value={text.get()} onChange={(value) => { text.set(clone(value)); }} className="grow min-h-0 text-[14px]" appearance={false} bordered={false} accessibilityLabel={message("Studio configuration")} />
    <View className="px-3 py-2 shrink-0"><Text>{localizeError(notice.get())}</Text><View hidden={!preserved.get()}><Text>{t("The existing settings file has been preserved.")}</Text></View></View>
  </View>;
}

export function showSettings(font: Signal<int>, settings: Signal<StudioConfig>): void {
  match (configPath()) {
    Ok(path) => {
      openWindow(SettingsEditor(path, font, settings), translate('Tsrust Studio Settings'), {
        width: 820, height: 640, minWidth: 600, minHeight: 400
      });
    },
    Err(error) => {
      openWindow(<View className="p-4"><Text>{localizeError(error)}</Text></View>, translate('Studio Settings Error'), {
        width: 600, height: 180
      });
    }
  }
}
