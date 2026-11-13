import { stat } from 'std:fs';
import { join } from 'std:path';
import {
  View,
  Text,
  Button,
  Command,
  CommandItem,
  CommandEmpty,
  Dialog,
  DialogContent,
  Tag,
  Node,
  Signal,
  computed,
  controlInput,
  pickFiles,
  translate,
  t
} from 'std:ui';

import { ContextFragment } from '../../lib/ai/conversations.tsr';
import { Entry, projectFiles } from '../../lib/workspace/workspace.tsr';
import { message, accessibleName } from '../../lib/localization.tsr';
import { AiContext, aiContextHandles, performModelAction } from '../../services/ai/actions.tsr';

export interface ContextPickerState {
  open: boolean;
  kind: string;
  query: string;
  chat: string;
  root: string;
  files: Entry[];
}

export function showContextPicker(context: ReadOnly<AiContext>, picker: Signal<ContextPickerState>, kind: string): void {
  const workspace = context.workspace;

  if (!context.allowRead || workspace === undefined) return;
  const root = clone(workspace.get().root);
  const directory = kind === 'skill' ? join(root, '.agents', 'skills') : clone(root);
  let files: Entry[] = [];

  match (stat(directory)) {
    Ok(_) => {
      match (projectFiles(directory, kind === 'skill')) {
        Ok(found) => {
          files = found.filter((file) => kind !== 'skill' || file.name === 'SKILL.md');
        },
        Err(error) => {
          const next = context.state.get();
          next.notice = clone(error);
          context.state.set(next);
          return;
        }
      }
    },
    Err(error) => {
      if (error.code !== 'not-found') return;
    }
  }

  picker.set({
    open: true, kind, query: '', chat: clone(context.state.get().history.active), root, files
  });
  controlInput('chat-context-search', 'focus');
}

function ContextOption(file: ReadOnly<Entry>, root: ReadOnly<string>): Node {
  return (
    <CommandItem value={clone(file.path)} label={file.path.slice(root.length + 1)} icon="file-code" />
  );
}

export function ContextPicker(context: ReadOnly<AiContext>, picker: Signal<ContextPickerState>, font: Signal<int>): Node {
  const select = aiContextHandles(context);
  const choose = aiContextHandles(context);
  const style = computed<string>(() => 'w-full min-h-0 text-[' + font.get().toString() + 'px]');
  return (
    <Dialog
      open={picker.get().open}
      title={picker.get().kind === 'skill' ? translate('Add skill instructions') : translate('Attach text files')}
      width={640}
      maxWidth={720}
      marginTop={64}
      overlayClosable={true}
      onOpenChange={(open) => {
        const next = picker.get();
        next.open = open;
        picker.set(next);
      }}
    >
      <DialogContent className="p-0">
        <Command
          className={style.get()}
          inputId="chat-context-search"
          filterable={false}
          query={picker.get().query}
          placeholder={message('Search context files')}
          maxHeight={320}
          onQuery={(query) => {
            const next = picker.get();
            next.query = clone(query);
            picker.set(next);
          }}
          onSelect={(path) => {
            performModelAction(select, 'chat_attach_context', JSON.stringify({
              chat: picker.get().chat, path: clone(path), kind: picker.get().kind
            }));
            const next = picker.get();
            next.open = false;
            picker.set(next);
          }}
          onCancel={() => {
            const next = picker.get();
            next.open = false;
            picker.set(next);
          }}
        >
          {...picker.get().files.filter((file: ReadOnly<Entry>) => file.path.toLowerCase().includes(picker.get().query.toLowerCase())).map((file: ReadOnly<Entry>) => ContextOption(file, picker.get().root))}
          <CommandEmpty>
            <Text className="text-muted-foreground">
              {t('No matching context files')}
            </Text>
          </CommandEmpty>
        </Command>
        <View className="flex-row px-2 py-1">
          <Button icon="folder-open" variant="ghost" size="small" onPress={() => {
            const chat=clone(picker.get().chat);
            const kind=clone(picker.get().kind);
            const multiple=kind !== 'skill';
            const scope=aiContextHandles(choose);
            pickFiles((paths) => {
              if (paths === '') return;
              const grant=aiContextHandles(scope);
              grant.allowedContextPaths=paths.split('\n');
              for (const path of grant.allowedContextPaths ?? []) performModelAction(grant,'chat_attach_context',JSON.stringify({chat:clone(chat),path:clone(path),kind:clone(kind)}));
              const next=picker.get();next.open=false;picker.set(next);
            },{multiple});
          }}>{t('Choose from disk…')}</Button>
        </View>
      </DialogContent>
    </Dialog>
  );
}

export function ContextChip(fragment: ReadOnly<ContextFragment>, context: ReadOnly<AiContext>, font: Signal<int>): Node {
  const remove = aiContextHandles(context);
  const chat = clone(context.state.get().history.active);
  const id = clone(fragment.id);
  const label = clone(fragment.label);
  const key = 'context-' + chat + '-' + fragment.id;
  const style = computed<string>(() => 'flex-row items-center gap-1 min-w-0 max-w-[180px] text-[' + font.get().toString() + 'px]');
  return (
    <View id={key}>
      <Tag size="small" className={style.get()}>
        <Text className="truncate min-w-0">
          {label}
        </Text>
        <Button
          icon="x"
          variant="ghost"
          size="xsmall"
          tooltip={message('Remove context')}
          aria-label={accessibleName('Remove context')}
          onPress={() => {
            performModelAction(remove, 'chat_detach_context', JSON.stringify({
              chat: clone(chat), context: clone(id)
            }));
          }}
        />
      </Tag>
    </View>
  );
}
