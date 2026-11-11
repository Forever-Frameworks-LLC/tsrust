import { View, Text, Terminal, Button, Node, Signal, signal, computed, readTerminal } from 'std:ui';
import { TerminalAuthority, TerminalArguments, invokeTerminalTool } from '../../services/terminal-actions.tsr';
import { message, accessibleName, localizeError } from '../../lib/localization.tsr';

function humanTerminalAction(id: string, name: string, error: Signal<string>, exited: Signal<boolean>): void {
  const restart = name === 'restart';
  const toolName = 'terminal_' + name;
  readTerminal(clone(id), (snapshot) => {
    const authority: TerminalAuthority = {
      sessionId: clone(id), allowRead: true, allowInput: true, allowNavigation: true, allowClipboard: true, allowTerminate: true
    };
    const args: TerminalArguments = {
      id: clone(id), generation: snapshot.generation ?? 0
    };
    invokeTerminalTool(authority, clone(toolName), JSON.stringify(args), (result) => {
      error.set(clone(result.error));
      if (result.ok && restart) exited.set(false);
    });
  });
}

export function TerminalPanel(directory: string, font: Signal<int>): Node {
  const error = signal('');
  const exited = signal(false);
  const panelClass = computed<string>(() => 'flex-col size-full min-h-0 text-[' + font.get().toString() + 'px]');
  return <View className={panelClass.get()}>
    <View className="flex-row justify-end items-center shrink-0 border-b border-border px-2 h-7 gap-1">
      <View hidden={!exited.get()}><Button icon="plus" variant="ghost" size="xsmall" tooltip={message("Restart exited shell")} aria-label={accessibleName("Restart exited shell")} onPress={() => { humanTerminalAction('workspace-terminal', 'restart', error, exited); }} /></View>
      <Button icon="square" variant="ghost" size="xsmall" tooltip={message("Interrupt terminal (Ctrl+C)")} aria-label={accessibleName("Interrupt terminal")} onPress={() => { humanTerminalAction('workspace-terminal', 'interrupt', error, exited); }} />
      <Button icon="copy" variant="ghost" size="xsmall" tooltip={message("Copy terminal selection")} aria-label={accessibleName("Copy terminal selection")} onPress={() => { humanTerminalAction('workspace-terminal', 'copy', error, exited); }} />
    </View>
    <Terminal id="workspace-terminal" directory={directory} fontSize={font.get()} onExit={() => { exited.set(true); }} accessibilityLabel={message("Workspace terminal")} className="grow min-h-0" />
    <View hidden={error.get() === ''} className="shrink-0 px-2"><Text className="text-destructive">{localizeError(error.get())}</Text></View>
  </View>;
}
