import {
  View,
  Text,
  Button,
  Textarea,
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  MessageScroller,
  Markdown,
  List,
  ListItem,
  ListItemSuffix,
  ListEmpty,
  Dialog,
  DialogContent,
  DialogFooter,
  Node,
  Signal,
  signal,
  computed,
  translate,
  t
} from 'std:ui';

import {
  AiMessage,
  Conversation,
  ContextFragment,
  activeMessages,
  contextTrigger,
  chatDraft,
  draftContext
} from '../../lib/ai/conversations.tsr';
import { accessibleName, message, localizeError } from '../../lib/localization.tsr';
import { AiState, AiContext, aiContextHandles, performModelAction as perform } from '../../services/ai/actions.tsr';

import { ContextPickerState, ContextPicker, ContextChip, showContextPicker } from './context-picker.tsrx';

interface MessageTarget {
  chat: string;
  request: string;
  role: 'user' | 'assistant';
}

function messageSnapshot(state: ReadOnly<AiState>, target: ReadOnly<MessageTarget>): AiMessage {
  const chat = state.history.chats.findIndex((chat) => chat.id === target.chat);

  if (chat >= 0) {
    const message = state.history.chats[chat].messages.findIndex((message) => message.request === target.request && message.role === target.role);

    if (message >= 0) return clone(state.history.chats[chat].messages[message]);
  }

  return {
    role: clone(target.role), request: clone(target.request), text: '', status: ''
  };
}

function ChatMessageView(entry: ReadOnly<AiMessage>, state: Signal<AiState>, font: Signal<int>): Node {
  const pendingRequest = clone(entry.request);
  const sent = entry.role === 'user';
  const chat = clone(state.get().history.active);
  const key = 'chat-' + chat + '-' + entry.request + '-' + entry.role;
  const target: MessageTarget = {
    chat, request: clone(entry.request), role: clone(entry.role)
  };
  const statusTarget = clone(target);
  const limitedTarget = clone(target);
  const contextTarget = clone(target);
  const text = computed<string>(() => messageSnapshot(state.get(), target).text);
  const delivery = computed<string>(() => messageSnapshot(state.get(), statusTarget).status ?? '');
  const limited = computed<boolean>(() => messageSnapshot(state.get(), limitedTarget).limited ?? false);
  const contextLabel = computed<string>(() => {
    const fragments: ContextFragment[] = messageSnapshot(state.get(), contextTarget).context ?? [];
    return fragments.map((fragment) => clone(fragment.label)).join(' · ');
  });
  const rowClass = computed<string>(() => 'flex-row w-full min-w-0 py-2 text-[' + font.get().toString() + 'px] ' + (sent ? 'justify-end' : 'justify-start'));
  const bodyClass = sent ? 'max-w-[90%] min-w-0 px-3 py-1 bg-muted rounded-md' : 'max-w-full min-w-0 py-1';
  const markdownClass = computed<string>(() => 'min-w-0 text-[' + font.get().toString() + 'px]');
  return (
    <View id={key} className={rowClass.get()}>
      <View className={bodyClass}>
        <Markdown content={text.get()} className={markdownClass.get()} />
        <View hidden={contextLabel.get() === ''}>
          <Text className="text-muted-foreground">
            {contextLabel.get()}
          </Text>
        </View>
        <View hidden={sent || text.get() !== '' || state.get().active !== pendingRequest}>
          <Text className="text-muted-foreground">
            {t('Preparing response…')}
          </Text>
        </View>
        <View hidden={sent || delivery.get() !== 'interrupted'}>
          <Text className="text-muted-foreground">
            {t('Response interrupted when Studio closed.')}
          </Text>
        </View>
        <View hidden={sent || delivery.get() !== 'cancelled'}>
          <Text className="text-muted-foreground">
            {t('Response stopped.')}
          </Text>
        </View>
        <View hidden={sent || delivery.get() !== 'complete' || text.get() !== ''}>
          <Text className="text-muted-foreground">
            {t('The model returned an empty response. Try another prompt.')}
          </Text>
        </View>
        <View hidden={sent || limited.get() !== true}>
          <Text className="text-muted-foreground">
            {t('Generation reached its available token limit.')}
          </Text>
        </View>
      </View>
    </View>
  );
}

interface DeleteChatTarget {
  chat: string;
  revision: int;
}

function ConversationRow(chat: ReadOnly<Conversation>, state: Signal<AiState>, font: Signal<int>, deleting: Signal<DeleteChatTarget>): Node {
  const id = clone(chat.id);
  const deleteId = clone(id);
  const captionId = clone(id);
  const targetId = clone(id);
  const title = computed<string>(() => {
    const index = state.get().history.chats.findIndex((chat) => chat.id === captionId);
    return index < 0 || state.get().history.chats[index].title === '' ? translate('New chat') : clone(state.get().history.chats[index].title);
  });
  const style = computed<string>(() => 'py-2 rounded-none border-b border-border text-[' + font.get().toString() + 'px]');
  return (
    <ListItem value={id} accessibilityLabel={title.get()} className={style.get()}>
      <Text className="grow min-w-0 truncate">
        {title.get()}
      </Text>
      <ListItemSuffix>
        <Button
          icon="trash"
          variant="ghost"
          size="xsmall"
          tooltip={message('Delete chat')}
          aria-label={accessibleName('Delete chat')}
          disabled={state.get().targetChat === targetId && state.get().active !== ''}
          onPress={() => {
            const current = state.get();
            const index = current.history.chats.findIndex((chat) => chat.id === deleteId);

            if (index >= 0) deleting.set({
              chat: clone(deleteId), revision: current.history.chats[index].revision
            });
          }}
        />
      </ListItemSuffix>
    </ListItem>
  );
}

function NewChatButton(context: ReadOnly<AiContext>): Node {
  const action = aiContextHandles(context);
  return (
    <Button
      icon="plus"
      variant="ghost"
      size="xsmall"
      tooltip={message('New chat')}
      aria-label={accessibleName('New chat')}
      onPress={() => {
        perform(action, 'chat_new');
      }}
    />
  );
}

function status(state: ReadOnly<AiState>): string {
  if (state.phase === 'starting') return translate('Loading model…');

  if (state.phase === 'generating') return translate('Responding…');

  if (state.phase === 'stopping') return translate('Stopping…');

  if (state.phase === 'ready') return translate('Ready');
  return translate('Starts automatically when you send');
}

export function ChatPanel(context: ReadOnly<AiContext>, font: Signal<int>, state: Signal<AiState>): Node {
  const picker = signal<ContextPickerState>({
    open: false, kind: 'file', query: '', chat: '', root: '', files: []
  });
  const deleting = signal<DeleteChatTarget>({
    chat: '', revision: 0
  });
  const input = aiContextHandles(context);
  const submit = aiContextHandles(context);
  const composer = aiContextHandles(context);
  const navigation = aiContextHandles(context);
  const open = aiContextHandles(context);
  const remove = aiContextHandles(context, true);
  const attach = aiContextHandles(context);
  const skills = aiContextHandles(context);
  const chip = aiContextHandles(context);
  const renderContext = (fragment: ReadOnly<ContextFragment>): Node => ContextChip(fragment, chip, font);
  const renderMessage = (entry: ReadOnly<AiMessage>): Node => ChatMessageView(entry, state, font);
  const renderChat = (chat: ReadOnly<Conversation>): Node => ConversationRow(chat, state, font, deleting);
  const style = computed<string>(() => 'flex-col size-full min-w-0 min-h-0 bg-background text-[' + font.get().toString() + 'px]');
  const inputStyle = computed<string>(() => 'min-w-0 text-[' + font.get().toString() + 'px]');
  const working = computed<boolean>(() => state.get().active !== '' || state.get().phase === 'stopping');
  return (
    <View className={style.get()}>
      <View hidden={state.get().view !== 'all'} className="flex-col grow min-h-0">
        <View className="flex-row items-center px-3 py-2">
          <Text className="grow">
            {t('Chats')}
          </Text>
          {NewChatButton(context)}
        </View>
        <List
          className="grow min-h-0"
          size="small"
          onConfirm={(id) => {
            perform(open, 'chat_open', JSON.stringify({
              chat: clone(id)
            }));
          }}
        >
          {...state.get().history.chats.map(renderChat)}
          <ListEmpty>
            <Text className="text-muted-foreground">
              {t('No conversations yet')}
            </Text>
          </ListEmpty>
        </List>
      </View>
      <View hidden={state.get().view !== 'thread'} className="flex-col grow min-h-0">
        <View className="flex-row items-center shrink-0 px-1 h-8">
          <Button
            icon="arrow-left"
            variant="ghost"
            size="xsmall"
            tooltip={message('All chats')}
            aria-label={accessibleName('All chats')}
            onPress={() => {
              perform(navigation, 'chat_show_all');
            }}
          />
        </View>
        <MessageScroller className="grow min-h-0" contentClassName="py-2" rowClassName="px-3 pb-1">
          {...activeMessages(state.get().history).map(renderMessage)}
        </MessageScroller>
        <View className="w-full shrink-0 min-w-0 flex-col overflow-hidden">
          <View hidden={state.get().notice === ''} className="px-1 py-2">
            <Text className="text-destructive">
              {localizeError(state.get().notice)}
            </Text>
          </View>
          <InputGroup size="small" className={'w-full overflow-hidden rounded-none border-0 border-t ' + inputStyle.get()} accessibilityLabel={message('AI composer')}>
            <Textarea
              value={chatDraft(state.get().history)}
              placeholder={message('Ask anything')}
              aria-label={accessibleName('Ask anything')}
              className={inputStyle.get()}
              minRows={2}
              maxRows={8}
              submitOnEnter={true}
              onChange={(event) => {
                const text = clone(event.target.value);
                const kind = contextTrigger(text);
                const prompt = kind === '' ? clone(text) : text.slice(0, -1);

                if (perform(input, 'chat_set_draft', JSON.stringify({
                  chat: state.get().history.active, prompt
                })) && kind !== '') showContextPicker(input, picker.handle(), kind);
              }}
              onSubmit={(_secondary) => {
                if (!working.get()) perform(submit, 'local_model_send', JSON.stringify({
                  chat: state.get().history.active, prompt: chatDraft(state.get().history)
                }));
              }}
            />
            <InputGroupAddon align="block-start" className={draftContext(state.get().history).length === 0 ? 'h-0 p-0 overflow-hidden' : 'min-w-0 gap-1'}>
              <View className="flex-row flex-wrap gap-1 min-w-0">
                {...draftContext(state.get().history).map(renderContext)}
              </View>
            </InputGroupAddon>
            <InputGroupAddon align="block-end" className="min-w-0 gap-1">
              <InputGroupButton
                icon="paperclip"
                tooltip={translate('Attach text files')}
                accessibilityLabel={translate('Attach text files')}
                onClick={() => {
                  showContextPicker(attach, picker.handle(), 'file');
                }}
              />
              <InputGroupButton
                icon="book-open"
                tooltip={translate('Add skill instructions')}
                accessibilityLabel={translate('Add skill instructions')}
                onClick={() => {
                  showContextPicker(skills, picker.handle(), 'skill');
                }}
              />
              <View className="grow min-w-0" />
              <InputGroupButton
                icon={working.get() ? 'square' : 'arrow-up'}
                disabled={state.get().phase === 'stopping' || (!working.get() && chatDraft(state.get().history).trim() === '')}
                tooltip={working.get() ? translate('Stop response') : translate('Send message')}
                accessibilityLabel={working.get() ? translate('Stop response') : translate('Send message')}
                onClick={() => {
                  if (working.get()) perform(composer, 'local_model_cancel', JSON.stringify({
                    request: state.get().active
                  }));
                  else perform(composer, 'local_model_send', JSON.stringify({
                    chat: state.get().history.active, prompt: chatDraft(state.get().history)
                  }));
                }}
              />
            </InputGroupAddon>
          </InputGroup>
          <Text className="px-1 pt-1 text-muted-foreground">
            {status(state.get())}
          </Text>
        </View>
      </View>
      {ContextPicker(context, picker.handle(), font.handle())}
      <Dialog
        open={deleting.get().chat !== ''}
        title={message('Delete this chat?')}
        description={message('This removes the conversation and its messages.')}
        width={420}
        onOpenChange={(open) => {
          if (!open) deleting.set({
            chat: '', revision: 0
          });
        }}
      >
        <DialogContent>
          <View hidden={state.get().notice === ''}>
            <Text className="text-destructive">
              {localizeError(state.get().notice)}
            </Text>
          </View>
          <DialogFooter>
            <Button
              variant="ghost"
              onPress={() => {
                deleting.set({
                  chat: '', revision: 0
                });
              }}
            >
              {t('Cancel')}
            </Button>
            <Button
              variant="danger"
              onPress={() => {
                if (perform(remove, 'chat_delete', JSON.stringify(deleting.get()))) deleting.set({
                  chat: '', revision: 0
                });
              }}
            >
              {t('Delete chat')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </View>
  );
}
