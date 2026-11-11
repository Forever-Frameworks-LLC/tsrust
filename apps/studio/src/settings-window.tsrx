import { Workspace } from './workspace.tsr';
import { message, accessibleName, localizeError } from './localization.tsr';
import { View, Text, Button, CodeEditor, Select, SelectItem, NumberInput, Settings, SettingsPage, SettingsGroup, SettingItem, Shortcut, Node, Signal, signal, openWindow, translate } from 'std:ui';
import { readTextFile } from 'std:fs';
import { toInt } from 'std:numeric';
import { configPath, parseConfig, formatConfig, defaultConfig, themes, StudioConfig } from './settings.tsr';
import { SettingsSession, commitSettings, saveSettings, reloadSettings, showSettingsControls } from './settings-actions.tsr';

function SettingsEditor(path: string, font: Signal<int>, settings: Signal<StudioConfig>, workspace: Signal<Workspace>): Node {
  let initial = defaultConfig();
  let initialText = formatConfig(initial);
  let priorText = '';
  let found = false;
  let readable = true;
  const notice = signal('');
  const raw = signal(false);
  match (readTextFile(path, 1024 * 1024)) {
    Ok(text) => {
      initialText = clone(text);
      priorText = clone(text);
      found = true;
      match (parseConfig(text)) {
        Ok(config) => {
          initialText = formatConfig(config);
          initial = config;
        },
        Err(error) => {
          notice.set(error);
          raw.set(true);
        }
      }
    },
    Err(error) => {
      if (error.code !== 'not-found') {
        notice.set(clone(error.message));
        raw.set(true);
        readable = false;
      }
    }
  }

  const draft = signal(initial);
  const text = signal(initialText);
  const baseline = signal(priorText);
  const present = signal(found);
  const ready = signal(readable);
  const controlsSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const saveSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const shortcutSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const reloadSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const themeSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const fontSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const localeSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const navigationSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const activitySession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const panelSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const chatSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  return <View className="flex-col size-full min-h-0 bg-background text-foreground">
    <View windowDragRegion={true} className="relative shrink-0 h-9 bg-muted border-b border-border">
      <View className="absolute left-20 top-0 bottom-0 flex-row items-center gap-1">
        <Button icon="settings" variant="ghost" size="small" selected={!raw.get()} aria-label={accessibleName("Settings controls")} tooltip={message("Settings controls")} onPress={() => { showSettingsControls(controlsSession); }} />
        <Button icon="braces" variant="ghost" size="small" selected={raw.get()} aria-label={accessibleName("Edit settings JSON")} tooltip={message("Edit settings JSON")} onPress={() => { raw.set(true); }} />
      </View>
      <View className="absolute right-2 top-0 bottom-0 flex-row items-center gap-1">
        <View hidden={!raw.get()}><Button icon="save" variant="ghost" size="small" aria-label={accessibleName("Save settings")} tooltip={message("Save settings")} onPress={() => { saveSettings(saveSession); }} /></View>
        <Button icon="rotate-ccw" variant="ghost" size="small" aria-label={accessibleName("Reload settings")} tooltip={message("Reload settings")} onPress={() => { reloadSettings(reloadSession); }} />
      </View>
    </View>
    <View hidden={raw.get()} className="flex-col grow min-h-0">
      <Settings sidebarWidth={210} sidebarMinWidth={160} sidebarMaxWidth={300} size="small" className="size-full min-h-0">
        <SettingsPage title={message("General")} icon="settings" resettable={false}>
          <SettingsGroup title={message("Appearance")}>
            <SettingItem title={message("Theme")} description={message("Choose the editor and workbench palette.")}>
              <Select items={themes()} value={draft.get().theme} searchable={true} accessibilityLabel={message("Color theme")} size="small" className="w-56" onChange={(name) => { const config = draft.get(); config.theme = clone(name); commitSettings(themeSession, config); }} />
            </SettingItem>
            <SettingItem title={message("Font size")} description={message("Base text size in pixels. Workspace zoom is applied on top.")}>
              <NumberInput value={Number(draft.get().fontSize)} min={8} max={48} step={1} onValidate={(candidate) => Number.isInteger(Number(candidate))} size="small" className="w-28" accessibilityLabel={message("Font size")} onChange={(value) => { match (toInt(value)) { Ok(size) => { const config = draft.get(); config.fontSize = size; commitSettings(fontSession, config); }, Err(_) => {} } }} />
            </SettingItem>
          </SettingsGroup>
          <SettingsGroup title={message("Language")}>
            <SettingItem title={message("Interface language")} description={message("Apply translated controls without restarting.")}>
              <Select value={draft.get().locale} accessibilityLabel={message("Language")} size="small" className="w-56" onChange={(value) => { const config = draft.get(); config.locale = clone(value); commitSettings(localeSession, config); }}><SelectItem value="en" label="English" /><SelectItem value="pt-BR" label="Português (Brasil)" /></Select>
            </SettingItem>
          </SettingsGroup>
        </SettingsPage>
        <SettingsPage title={message("Workbench")} icon="panels-top-left" resettable={false}>
          <SettingsGroup title={message("Navigation")}>
            <SettingItem title={message("File explorer position")} description={message("Place the file tree on either side of the editor.")}>
              <Select value={draft.get().navigationSide} accessibilityLabel={message("File explorer position")} size="small" className="w-40" onChange={(value) => { const config = draft.get(); config.navigationSide = clone(value); commitSettings(navigationSession, config); }}><SelectItem value="left" label={message("Left")} /><SelectItem value="right" label={message("Right")} /></Select>
            </SettingItem>
            <SettingItem title={message("Activity bar position")}>
              <Select value={draft.get().activityBarSide} accessibilityLabel={message("Activity bar position")} size="small" className="w-40" onChange={(value) => { const config = draft.get(); config.activityBarSide = clone(value); commitSettings(activitySession, config); }}><SelectItem value="left" label={message("Left")} /><SelectItem value="right" label={message("Right")} /></Select>
            </SettingItem>
          </SettingsGroup>
          <SettingsGroup title={message("Panels")}>
            <SettingItem title={message("Terminal position")}>
              <Select value={draft.get().panelPosition} accessibilityLabel={message("Terminal position")} size="small" className="w-40" onChange={(value) => { const config = draft.get(); config.panelPosition = clone(value); commitSettings(panelSession, config); }}><SelectItem value="bottom" label={message("Bottom")} /><SelectItem value="left" label={message("Left")} /><SelectItem value="right" label={message("Right")} /></Select>
            </SettingItem>
            <SettingItem title={message("AI chat position")}>
              <Select value={draft.get().chatPosition} accessibilityLabel={message("AI chat position")} size="small" className="w-40" onChange={(value) => { const config = draft.get(); config.chatPosition = clone(value); commitSettings(chatSession, config); }}><SelectItem value="right" label={message("Right")} /><SelectItem value="left" label={message("Left")} /><SelectItem value="bottom" label={message("Bottom")} /></Select>
            </SettingItem>
          </SettingsGroup>
        </SettingsPage>
      </Settings>
    </View>
    <View hidden={!raw.get()} className="flex-col grow min-h-0">
      <CodeEditor language="json" value={text.get()} onChange={(value) => { text.set(clone(value)); }} className="size-full min-h-0 font-mono text-[14px]" appearance={false} bordered={false} accessibilityLabel={message("Studio configuration")} />
    </View>
    <View hidden={notice.get() === ''} className="px-3 py-2 shrink-0 border-t border-border"><Text className="text-destructive">{localizeError(notice.get())}</Text></View>
    <Shortcut command="save-file" keys="secondary-s" onTrigger={() => { if (raw.get()) saveSettings(shortcutSession); }} />
  </View>;
}

export function showSettings(font: Signal<int>, settings: Signal<StudioConfig>, workspace: Signal<Workspace>): void {
  match (configPath()) {
    Ok(path) => {
      openWindow(SettingsEditor(path, font, settings, workspace), translate('Tsrust Studio Settings'), {
        titlebar: 'custom', width: 900, height: 650, minWidth: 680, minHeight: 480
      });
    },
    Err(error) => {
      openWindow(<View className="flex-col size-full"><View windowDragRegion={true} className="h-9 shrink-0 bg-muted border-b border-border" /><View className="p-4"><Text>{localizeError(error)}</Text></View></View>, translate('Studio Settings Error'), {
        titlebar: 'custom', width: 600, height: 180
      });
    }
  }
}
