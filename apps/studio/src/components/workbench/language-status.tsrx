import { Channel, Sender } from 'std:futures';
import {
  View,
  Button,
  DropdownMenu,
  MenuItem,
  Submenu,
  Ticker,
  Node,
  Signal,
  computed,
  translate
} from 'std:ui';

import { languageRoute } from '../../lib/language/servers.tsr';
import { Problem } from '../../lib/workspace/problems.tsr';
import { Workspace } from '../../lib/workspace/workspace.tsr';
import { requestLanguageControl, languageControlAvailable } from '../../services/lsp/actions.tsr';
import { LanguageState, LanguageConnection, connectionFor, pollLanguage } from '../../services/lsp/controller.tsr';
import { LanguageMessage, LanguageUpdate } from '../../services/lsp/session.tsr';

function selectedConnection(workspace: ReadOnly<Workspace>, state: ReadOnly<LanguageState>): LanguageConnection | undefined {
  const route = languageRoute(state.servers, workspace.active);
  return connectionFor(state, route?.server ?? 'tsrust');
}

function ConnectionAction(server: string, action: string, state: Signal<LanguageState>, messages: Sender<LanguageMessage>): Node {
  const enabledServer = clone(server);
  const enabledAction = clone(action);
  const enabled = computed<boolean>(() => {
    const connection = connectionFor(state.get(), enabledServer);
    return connection !== undefined && languageControlAvailable(connection, enabledAction);
  });
  const label = action === 'start' ? 'Start' : action === 'stop' ? 'Stop' : 'Restart';
  const icon = action === 'start' ? 'play' : action === 'stop' ? 'square' : 'rotate-cw';
  return (
    <MenuItem
      label={translate(label)}
      icon={icon}
      disabled={!enabled.get()}
      onSelect={() => {
        const next = state.get();
        const current = connectionFor(next, server);

        if (current === undefined) return;

        match (requestLanguageControl(next, messages, clone(server), current.generation, clone(action))) {
          Ok(_) => {
            state.set(next);
          },
          Err(_) => {}
        }
      }}
    />
  );
}

function ConnectionMenu(connection: ReadOnly<LanguageConnection>, state: Signal<LanguageState>, messages: Sender<LanguageMessage>): Node {
  const server = clone(connection.server);
  const labelServer = clone(server);
  const label = computed<string>(() => labelServer + ': ' + translate(connectionFor(state.get(), labelServer)?.status ?? 'LSP: idle'));
  return (
    <Submenu label={label.get()}>
      {ConnectionAction(clone(server), 'start', state, messages.share())}
      {ConnectionAction(clone(server), 'stop', state, messages.share())}
      {ConnectionAction(clone(server), 'restart', state, messages.share())}
    </Submenu>
  );
}

export function LanguageStatus(
  workspace: Signal<Workspace>,
  problems: Signal<Problem[]>,
  panel: Signal<string>,
  font: Signal<int>,
  messages: Sender<LanguageMessage>,
  restarts: Sender<LanguageMessage>,
  updates: Channel<LanguageUpdate>,
  state: Signal<LanguageState>
): Node {
  const renderConnection = (connection: ReadOnly<LanguageConnection>): Node => ConnectionMenu(connection, state, restarts.share());
  const label = computed<string>(() => translate(selectedConnection(workspace.get(), state.get())?.status ?? 'LSP: idle'));
  const detail = computed<string>(() => selectedConnection(workspace.get(), state.get())?.detail ?? '');
  return (
    <View>
      <DropdownMenu fontSize={font.get()}>
        <Button icon="languages" variant="ghost" size="xsmall" tooltip={label.get() + ' — ' + detail.get()} aria-label={translate('Language services')} />
        <MenuItem
          label={translate('Show language diagnostics')}
          onSelect={() => {
            panel.set('problems');
          }}
        />
        {...state.get().connections.map(renderConnection)}
      </DropdownMenu>
      <Ticker
        interval={150}
        onTick={(_frame) => {
          pollLanguage(workspace, problems, state, messages, updates);
        }}
      />
    </View>
  );
}
