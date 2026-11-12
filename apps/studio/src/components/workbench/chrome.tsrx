import {
  View,
  Button,
  Text,
  computed,
  controlInput,
  t,
  Shortcut,
  AppMenuBar,
  AppMenu,
  MenuItem,
  Signal,
  Node
} from 'std:ui';

import { FileSearchState } from '../../lib/explorer/search-actions.tsr';
import { accessibleName, message } from '../../lib/localization.tsr';
import { StudioConfig } from '../../lib/settings/settings.tsr';
import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { canFormat } from '../../lib/workspace/formatting-actions.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { Workspace, closeFile } from '../../lib/workspace/workspace.tsr';
import { zoomWorkspace } from '../../lib/workspace/zoom.tsr';
import { unsavedTargets } from '../../services/save-actions.tsr';
import { openPalette } from '../explorer/navigation.tsrx';
import { showSettings } from '../settings/settings-window.tsrx';

export function closeTab(workspace: Signal<Workspace>, recovery: Signal<Recovery>, path: string): void {
  const next = workspace.get();

  match (closeFile(next, path)) {
    Ok(_) => {},
    Err(message) => {
      next.error = clone(message);
    }
  }

  publishWorkspace(next, workspace, recovery);
}

export function Chrome(
  workspace: Signal<Workspace>,
  sidebar: Signal<boolean>,
  mode: Signal<string>,
  font: Signal<int>,
  panel: Signal<string>,
  recovery: Signal<Recovery>,
  settings: Signal<StudioConfig>,
  fileSearch: Signal<FileSearchState>,
  chat: Signal<boolean>
): Node {
  const paletteClass = computed<string>(() => 'w-full max-w-[320px] h-6 px-3 gap-2 min-w-0 justify-start border border-border rounded-sm bg-background text-muted-foreground text-[' + font.get().toString() + 'px]');
  // macOS retains its traffic-light controls in the leading 80 logical pixels.
  return (
    <View className="relative shrink-0 h-9 bg-muted border-b border-border" windowDragRegion={true}>
      <AppMenuBar>
        <AppMenu label={message("Application")}>
          <MenuItem command="open-settings" label={message("Settings")} />
        </AppMenu>
        <AppMenu label={message("File")}>
          <MenuItem command="find-file" label={message("Find project file")} />
          <MenuItem command="save-all" label={message("Save All")} disabled={unsavedTargets(workspace.get()).length === 0} />
          <MenuItem command="close-tab" label={message("Close Editor")} />
        </AppMenu>
        <AppMenu label={message("Edit")}>
          <MenuItem command="show-commands" label={message("Command Palette")} />
          <MenuItem command="format-document" label={message("Format document")} disabled={!canFormat(workspace.get().active)} />
        </AppMenu>
        <AppMenu label={message("Project")}>
          <MenuItem command="refresh-explorer" label={message("Refresh Explorer")} />
          <MenuItem command="format-project" label={message("Format project")} />
          <MenuItem command="check-document" label={message("Check document")} disabled={!canFormat(workspace.get().active)} />
          <MenuItem command="check-project" label={message("Check project")} />
        </AppMenu>
      </AppMenuBar>
      <View className="absolute left-20 top-0 bottom-0 flex-row items-center">
        <Button
          icon="panel-left"
          variant="ghost"
          size="small"
          aria-label={accessibleName("Toggle sidebar")}
          tooltip={message("Toggle sidebar")}
          tooltipCommand="toggle-sidebar"
          onPress={() => {
            sidebar.set(!sidebar.get());
          }}
        />
      </View>
      <View className="absolute left-32 right-12 top-0 bottom-0 flex-row items-center justify-center min-w-0">
        <Button
          icon="search"
          variant="ghost"
          size="small"
          className={paletteClass.get()}
          aria-label={accessibleName("Search files and commands")}
          onPress={() => {
            openPalette(workspace, recovery, fileSearch);
          }}
        >
          <Text className="grow min-w-0 truncate">
            {t("Search files and commands")}
          </Text>
        </Button>
      </View>
      <View className="absolute right-2 top-0 bottom-0 flex-row items-center gap-2">
        <Button
          icon="sparkles"
          variant="ghost"
          size="small"
          selected={chat.get()}
          aria-label={accessibleName('AI Chat')}
          tooltip={message('AI Chat')}
          onPress={() => {
            chat.set(!chat.get());
          }}
        />
      </View>
      <Shortcut
        command="toggle-sidebar"
        keys="secondary-b"
        onTrigger={() => {
          sidebar.set(!sidebar.get());
        }}
      />
      <Shortcut
        command="find-file"
        keys="secondary-p"
        onTrigger={() => {
          openPalette(workspace, recovery, fileSearch);
        }}
      />
      <Shortcut
        command="show-commands"
        keys="secondary-shift-p"
        onTrigger={() => {
          openPalette(workspace, recovery, fileSearch, true);
        }}
      />
      <Shortcut
        command="zoom-in"
        keys="secondary-="
        onTrigger={() => {
          zoomWorkspace(workspace, recovery, font, settings, 1);
        }}
      />
      <Shortcut
        command="zoom-in-plus"
        keys="secondary-+"
        onTrigger={() => {
          zoomWorkspace(workspace, recovery, font, settings, 1);
        }}
      />
      <Shortcut
        command="zoom-out"
        keys="secondary--"
        onTrigger={() => {
          zoomWorkspace(workspace, recovery, font, settings, -1);
        }}
      />
      <Shortcut
        command="zoom-reset"
        keys="secondary-0"
        onTrigger={() => {
          zoomWorkspace(
            workspace,
            recovery,
            font,
            settings,
            0,
            true
          );
        }}
      />
      <Shortcut
        command="toggle-terminal"
        keys="secondary-j"
        onTrigger={() => {
          panel.set(panel.get() === 'terminal' ? '' : 'terminal');
        }}
      />
      <Shortcut
        command="open-settings"
        keys="secondary-,"
        onTrigger={() => {
          showSettings(font, settings, workspace);
        }}
      />
      <Shortcut
        command="search-workspace"
        keys="secondary-shift-f"
        onTrigger={() => {
          mode.set('search');
          sidebar.set(true);
          controlInput('content-search', 'focus');
          controlInput('content-search', 'selectAll');
        }}
      />
      <Shortcut
        command="close-tab"
        keys="secondary-w"
        onTrigger={() => {
          closeTab(workspace, recovery, workspace.get().active);
        }}
      />
    </View>
  );
}
