import { Channel, Sender } from 'std:futures';
import {
  View,
  Button,
  DropdownMenu,
  MenuItem,
  Ticker,
  Node,
  Signal,
  computed,
  translate
} from 'std:ui';

import { languageRoute } from '../../lib/language/servers.tsr';
import { Problem } from '../../lib/workspace/problems.tsr';
import { Workspace } from '../../lib/workspace/workspace.tsr';
import { requestLanguageRestart } from '../../services/lsp/actions.tsr';
import { LanguageState, LanguageConnection, connectionFor, pollLanguage } from '../../services/lsp/controller.tsr';
import { LanguageMessage, LanguageUpdate } from '../../services/lsp/session.tsr';

function selectedConnection(workspace: ReadOnly<Workspace>, state: ReadOnly<LanguageState>): LanguageConnection | undefined {
  const route = languageRoute(state.servers, workspace.active);
  return connectionFor(state, route?.server ?? 'tsrust');
}

function ConnectionMenu(connection: ReadOnly<LanguageConnection>, state: Signal<LanguageState>, restarts: Sender<LanguageMessage>): Node {
  const server = clone(connection.server);
  const labelServer = clone(server);
  const label = computed<string>(() => {
    const current = connectionFor(state.get(), labelServer);
    return labelServer + ': ' + translate(current?.status ?? 'LSP: idle') + ' — ' + translate(current?.requested === true ? 'Restart language service' : 'Start language service');
  });
  return (
    <MenuItem
      label={label.get()}
      onSelect={() => {
        const next = state.get();
        const current = connectionFor(next, server);

        if (current === undefined) return;

        match (requestLanguageRestart(next, restarts, clone(server), current.generation)) {
          Ok(_) => {
            state.set(next);
          },
          Err(_) => {}
        }
      }}
    />
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
