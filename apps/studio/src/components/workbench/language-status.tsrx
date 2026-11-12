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

import { Problem } from '../../lib/workspace/problems.tsr';
import { Workspace } from '../../lib/workspace/workspace.tsr';
import { restartLanguageService } from '../../services/lsp/actions.tsr';
import { LanguageState, pollLanguage } from '../../services/lsp/controller.tsr';
import { LanguageMessage, LanguageUpdate } from '../../services/lsp/session.tsr';


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
  const label = computed<string>(() => translate(state.get().status));
  return (
    <View>
      <DropdownMenu fontSize={font.get()}>
        <Button variant="ghost" size="xsmall" className={'text-[' + font.get().toString() + 'px]'} tooltip={state.get().detail} aria-label={label.get()}>
          {label.get()}
        </Button>
        <MenuItem
          label={translate('Show language diagnostics')}
          onSelect={() => {
            panel.set('problems');
          }}
        />
        <MenuItem
          label={translate('Restart language service')}
          onSelect={() => {
            match (restartLanguageService(restarts, state.get().generation)) {
              Ok(_) => {
                const next = state.get();
                next.ready = false;
                next.status = 'LSP: restarting';
                state.set(next);
              },
              Err(_) => {}
            }
          }}
        />
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
