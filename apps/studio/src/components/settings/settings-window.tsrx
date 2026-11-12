import { readTextFile } from 'std:fs';
import { toInt } from 'std:numeric';
import {
  View,
  Text,
  Button,
  CodeEditor,
  Select,
  SelectItem,
  NumberInput,
  Switch,
  Settings,
  SettingsPage,
  SettingsGroup,
  SettingItem,
  Shortcut,
  Node,
  Signal,
  signal,
  openWindow,
  translate
} from 'std:ui';

import { message, accessibleName, localizeError } from '../../lib/localization.tsr';
import { SettingsSession, commitSettings, saveSettings, reloadSettings, showSettingsControls } from '../../lib/settings/settings-actions.tsr';
import {
  configPath,
  parseConfig,
  formatConfig,
  defaultConfig,
  themes,
  StudioConfig
} from '../../lib/settings/settings.tsr';
import { Workspace } from '../../lib/workspace/workspace.tsr';

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
  const iconsSession: SettingsSession = {
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
  const formatSaveSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };

  const formatFocusSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };

  const checkFocusSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const checkSaveSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const indentSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const organizeImportsSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const importWrapSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const argumentWrapSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const jsxReturnSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const jsxChildrenSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  const blockSpacingSession: SettingsSession = {
    path: clone(path), draft, text, baseline, present, ready, notice, raw, font, settings, workspace
  };
  return (
    <View className="flex-col size-full min-h-0 bg-background text-foreground">
      <View windowDragRegion={true} className="relative shrink-0 h-9 bg-muted border-b border-border">
        <View className="absolute left-20 top-0 bottom-0 flex-row items-center gap-1">
          <Button
            icon="settings"
            variant="ghost"
            size="small"
            selected={!raw.get()}
            aria-label={accessibleName("Settings controls")}
            tooltip={message("Settings controls")}
            onPress={() => {
              showSettingsControls(controlsSession);
            }}
          />
          <Button
            icon="braces"
            variant="ghost"
            size="small"
            selected={raw.get()}
            aria-label={accessibleName("Edit settings JSON")}
            tooltip={message("Edit settings JSON")}
            onPress={() => {
              raw.set(true);
            }}
          />
        </View>
        <View className="absolute right-2 top-0 bottom-0 flex-row items-center gap-1">
          <View hidden={!raw.get()}>
            <Button
              icon="save"
              variant="ghost"
              size="small"
              aria-label={accessibleName("Save settings")}
              tooltip={message("Save settings")}
              onPress={() => {
                saveSettings(saveSession);
              }}
            />
          </View>
          <Button
            icon="rotate-ccw"
            variant="ghost"
            size="small"
            aria-label={accessibleName("Reload settings")}
            tooltip={message("Reload settings")}
            onPress={() => {
              reloadSettings(reloadSession);
            }}
          />
        </View>
      </View>
      <View hidden={raw.get()} className="flex-col grow min-h-0">
        <Settings sidebarWidth={210} sidebarMinWidth={160} sidebarMaxWidth={300} size="small" className="size-full min-h-0">
          <SettingsPage title={message("General")} icon="settings" resettable={false}>
            <SettingsGroup title={message("Appearance")}>
              <SettingItem title={message("Theme")} description={message("Choose the editor and workbench palette.")}>
                <Select
                  items={themes()}
                  value={draft.get().theme}
                  searchable={true}
                  accessibilityLabel={message("Color theme")}
                  size="small"
                  className="w-56"
                  onChange={(name) => {
                    const config = draft.get();
                    config.theme = clone(name);
                    commitSettings(themeSession, config);
                  }}
                />
              </SettingItem>
              <SettingItem title={message("Font size")} description={message("Base text size in pixels. Workspace zoom is applied on top.")}>
                <NumberInput
                  value={Number(draft.get().fontSize)}
                  min={8}
                  max={48}
                  step={1}
                  onValidate={(candidate) => Number.isInteger(Number(candidate))}
                  size="small"
                  className="w-28"
                  accessibilityLabel={message("Font size")}
                  onChange={(value) => {
                    match (toInt(value)) {
                      Ok(size) => {
                        const config = draft.get();
                        config.fontSize = size;
                        commitSettings(fontSession, config);
                      },
                      Err(_) => {}
                    }
                  }}
                />
              </SettingItem>
              <SettingItem title={message("File icons")} description={message("Show language and file-type symbols in the explorer.")}>
                <Switch
                  checked={draft.get().fileIcons}
                  accessibilityLabel={message("File icons")}
                  onChange={(enabled) => {
                    const config = draft.get();
                    config.fileIcons = enabled;
                    commitSettings(iconsSession, config);
                  }}
                />
              </SettingItem>
            </SettingsGroup>
            <SettingsGroup title={message("Language")}>
              <SettingItem title={message("Interface language")} description={message("Apply translated controls without restarting.")}>
                <Select
                  value={draft.get().locale}
                  accessibilityLabel={message("Language")}
                  size="small"
                  className="w-56"
                  onChange={(value) => {
                    const config = draft.get();
                    config.locale = clone(value);
                    commitSettings(localeSession, config);
                  }}
                >
                  <SelectItem value="en" label="English" />
                  <SelectItem value="pt-BR" label="Português (Brasil)" />
                </Select>
              </SettingItem>
            </SettingsGroup>
          </SettingsPage>
          <SettingsPage title={message("Formatting")} icon="braces" resettable={false}>
            <SettingsGroup title={message("On save")}>
              <SettingItem title={message("Format on save")} description={message("Format Tsrust buffers before saving. Project editor settings take precedence.")}>
                <Switch
                  checked={draft.get().formatOnSave}
                  accessibilityLabel={message("Format on save")}
                  onChange={(enabled) => {
                    const config = draft.get();
                    config.formatOnSave = enabled;
                    commitSettings(formatSaveSession, config);
                  }}
                />
              </SettingItem>
              <SettingItem title={message("Check on save")} description={message("Check saved project files after a successful save.")}>
                <Switch
                  checked={draft.get().checkOnSave}
                  accessibilityLabel={message("Check on save")}
                  onChange={(enabled) => {
                    const config = draft.get();
                    config.checkOnSave = enabled;
                    commitSettings(checkSaveSession, config);
                  }}
                />
              </SettingItem>
            </SettingsGroup>
            <SettingsGroup title={message("On focus change")}>
              <SettingItem title={message("Format on focus change")} description={message("Format the buffer when its editor loses focus. Does not save the file.")}>
                <Switch
                  checked={draft.get().formatOnFocusChange}
                  accessibilityLabel={message("Format on focus change")}
                  onChange={(enabled) => {
                    const config = draft.get();
                    config.formatOnFocusChange = enabled;
                    commitSettings(formatFocusSession, config);
                  }}
                />
              </SettingItem>
              <SettingItem title={message("Check on focus change")} description={message("Check saved project files when an editor loses focus. Unsaved changes are not checked.")}>
                <Switch
                  checked={draft.get().checkOnFocusChange}
                  accessibilityLabel={message("Check on focus change")}
                  onChange={(enabled) => {
                    const config = draft.get();
                    config.checkOnFocusChange = enabled;
                    commitSettings(checkFocusSession, config);
                  }}
                />
              </SettingItem>
            </SettingsGroup>
            <SettingsGroup title={message("Formatting rules")}>
              <SettingItem title={message("Indent width")} description={message("Spaces per indentation level (1–8). Project format settings take precedence.")}>
                <NumberInput
                  value={Number(draft.get().format.indentWidth)}
                  min={1}
                  max={8}
                  step={1}
                  onValidate={(candidate) => Number.isInteger(Number(candidate))}
                  size="small"
                  className="w-28"
                  accessibilityLabel={message("Indent width")}
                  onChange={(value) => {
                    match (toInt(value)) {
                      Ok(width) => {
                        const config = draft.get();
                        config.format.indentWidth = width;
                        commitSettings(indentSession, config);
                      },
                      Err(_) => {}
                    }
                  }}
                />
              </SettingItem>
              <SettingItem title={message("Organize imports")} description={message("Sort imports by path: std: imports, code imports, then assets, with blank lines between groups.")}>
                <Switch
                  checked={draft.get().format.organizeImports}
                  accessibilityLabel={message("Organize imports")}
                  onChange={(enabled) => {
                    const config = draft.get();
                    config.format.organizeImports = enabled;
                    commitSettings(organizeImportsSession, config);
                  }}
                />
              </SettingItem>
              <SettingItem title={message("Wrap named imports above")} description={message("Wrap named imports onto separate lines above this count. Set 0 to disable (0–100). Inclusive of aliases.")}>
                <NumberInput
                  value={Number(draft.get().format.importWrapThreshold)}
                  min={0}
                  max={100}
                  step={1}
                  onValidate={(candidate) => Number.isInteger(Number(candidate))}
                  size="small"
                  className="w-28"
                  accessibilityLabel={message("Wrap named imports above")}
                  onChange={(value) => {
                    match (toInt(value)) {
                      Ok(count) => {
                        const config = draft.get();
                        config.format.importWrapThreshold = count;
                        commitSettings(importWrapSession, config);
                      },
                      Err(_) => {}
                    }
                  }}
                />
              </SettingItem>
              <SettingItem title={message("Wrap arguments above")} description={message("Wrap function parameters and call arguments onto separate lines above this count. Set 0 to disable (0–100).")}>
                <NumberInput
                  value={Number(draft.get().format.argumentWrapThreshold)}
                  min={0}
                  max={100}
                  step={1}
                  onValidate={(candidate) => Number.isInteger(Number(candidate))}
                  size="small"
                  className="w-28"
                  accessibilityLabel={message("Wrap arguments above")}
                  onChange={(value) => {
                    match (toInt(value)) {
                      Ok(count) => {
                        const config = draft.get();
                        config.format.argumentWrapThreshold = count;
                        commitSettings(argumentWrapSession, config);
                      },
                      Err(_) => {}
                    }
                  }}
                />
              </SettingItem>
              <SettingItem title={message("Parenthesize TSRX returns")}>
                <Switch
                  checked={draft.get().format.jsxReturnParentheses}
                  accessibilityLabel={message("Parenthesize TSRX returns")}
                  onChange={(enabled) => {
                    const config = draft.get();
                    config.format.jsxReturnParentheses = enabled;
                    commitSettings(jsxReturnSession, config);
                  }}
                />
              </SettingItem>
              <SettingItem title={message("Put component children on separate lines")}>
                <Switch
                  checked={draft.get().format.jsxChildrenOnNewLines}
                  accessibilityLabel={message("Put component children on separate lines")}
                  onChange={(enabled) => {
                    const config = draft.get();
                    config.format.jsxChildrenOnNewLines = enabled;
                    commitSettings(jsxChildrenSession, config);
                  }}
                />
              </SettingItem>
              <SettingItem title={message("Blank lines around control-flow blocks")}>
                <Switch
                  checked={draft.get().format.blankLinesAroundBlocks}
                  accessibilityLabel={message("Blank lines around control-flow blocks")}
                  onChange={(enabled) => {
                    const config = draft.get();
                    config.format.blankLinesAroundBlocks = enabled;
                    commitSettings(blockSpacingSession, config);
                  }}
                />
              </SettingItem>
            </SettingsGroup>
          </SettingsPage>
          <SettingsPage title={message("Workbench")} icon="panels-top-left" resettable={false}>
            <SettingsGroup title={message("Navigation")}>
              <SettingItem title={message("File explorer position")} description={message("Place the file tree on either side of the editor.")}>
                <Select
                  value={draft.get().navigationSide}
                  accessibilityLabel={message("File explorer position")}
                  size="small"
                  className="w-40"
                  onChange={(value) => {
                    const config = draft.get();
                    config.navigationSide = clone(value);
                    commitSettings(navigationSession, config);
                  }}
                >
                  <SelectItem value="left" label={message("Left")} />
                  <SelectItem value="right" label={message("Right")} />
                </Select>
              </SettingItem>
              <SettingItem title={message("Activity bar position")}>
                <Select
                  value={draft.get().activityBarSide}
                  accessibilityLabel={message("Activity bar position")}
                  size="small"
                  className="w-40"
                  onChange={(value) => {
                    const config = draft.get();
                    config.activityBarSide = clone(value);
                    commitSettings(activitySession, config);
                  }}
                >
                  <SelectItem value="left" label={message("Left")} />
                  <SelectItem value="right" label={message("Right")} />
                </Select>
              </SettingItem>
            </SettingsGroup>
            <SettingsGroup title={message("Panels")}>
              <SettingItem title={message("Terminal position")}>
                <Select
                  value={draft.get().panelPosition}
                  accessibilityLabel={message("Terminal position")}
                  size="small"
                  className="w-40"
                  onChange={(value) => {
                    const config = draft.get();
                    config.panelPosition = clone(value);
                    commitSettings(panelSession, config);
                  }}
                >
                  <SelectItem value="bottom" label={message("Bottom")} />
                  <SelectItem value="left" label={message("Left")} />
                  <SelectItem value="right" label={message("Right")} />
                </Select>
              </SettingItem>
              <SettingItem title={message("AI chat position")}>
                <Select
                  value={draft.get().chatPosition}
                  accessibilityLabel={message("AI chat position")}
                  size="small"
                  className="w-40"
                  onChange={(value) => {
                    const config = draft.get();
                    config.chatPosition = clone(value);
                    commitSettings(chatSession, config);
                  }}
                >
                  <SelectItem value="right" label={message("Right")} />
                  <SelectItem value="left" label={message("Left")} />
                  <SelectItem value="bottom" label={message("Bottom")} />
                </Select>
              </SettingItem>
            </SettingsGroup>
          </SettingsPage>
        </Settings>
      </View>
      <View hidden={!raw.get()} className="flex-col grow min-h-0">
        <CodeEditor
          language="json"
          value={text.get()}
          onChange={(value) => {
            text.set(clone(value));
          }}
          className="size-full min-h-0 font-mono text-[14px]"
          appearance={false}
          bordered={false}
          accessibilityLabel={message("Studio configuration")}
        />
      </View>
      <View hidden={notice.get() === ''} className="px-3 py-2 shrink-0 border-t border-border">
        <Text className="text-destructive">
          {localizeError(notice.get())}
        </Text>
      </View>
      <Shortcut
        command="save-file"
        keys="secondary-s"
        onTrigger={() => {
          if (raw.get()) saveSettings(shortcutSession);
        }}
      />
    </View>
  );
}

export function showSettings(font: Signal<int>, settings: Signal<StudioConfig>, workspace: Signal<Workspace>): void {
  match (configPath()) {
    Ok(path) => {
      openWindow(SettingsEditor(path, font, settings, workspace), translate('Tsrust Studio Settings'), {
        titlebar: 'custom', width: 900, height: 650, minWidth: 680, minHeight: 480
      });
    },
    Err(error) => {
      openWindow(
        <View className="flex-col size-full">
          <View windowDragRegion={true} className="h-9 shrink-0 bg-muted border-b border-border" />
          <View className="p-4">
            <Text>
              {localizeError(error)}
            </Text>
          </View>
        </View>, translate('Studio Settings Error'), {
        titlebar: 'custom', width: 600, height: 180
      });
    }
  }
}
