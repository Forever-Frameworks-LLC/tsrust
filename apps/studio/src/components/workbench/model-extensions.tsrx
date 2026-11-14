import { Channel, Sender } from 'std:futures';
import { basename, join } from 'std:path';
import { readTextFile, stat } from 'std:fs';
import {
  Node,
  Signal,
  Button,
  View,
  Text,
  Ticker,
  Input,
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  pickFilePaths,
  computed,
  translate,
  signal
} from 'std:ui';
import { accessibleName, message, localizeError } from '../../lib/localization.tsr';
import {
  StudioConfig,
  configPath,
  parseConfig,
  formatConfig,
  saveConfig,
  defaultConfig
} from '../../lib/settings/settings.tsr';
import { AiState } from '../../services/ai/actions.tsr';
import { ModelRequest } from '../../lib/ai/protocol.tsr';
import { ModelExtension, modelExtensions, availableModelPaths, modelTokenizer, modelFolder } from '../../services/ai/model-extensions.tsr';

export function toggleExtensions(sidebar: Signal<boolean>, mode: Signal<string>, settings: Signal<StudioConfig>, modelPaths: Signal<string[]>): void {
  sidebar.set(mode.get() !== 'extensions' || !sidebar.get());
  mode.set('extensions');

  if (sidebar.get()) modelPaths.set(availableModelPaths(settings.get()));
}

function ExtensionItem(name: string, label: string, selected: Signal<string>, font: Signal<int>, query: Signal<string>): Node {

  const selectedName = clone(name);
  const searchName = label.toLowerCase();
  const active = computed<boolean>(() => selected.get() === selectedName);
  const style = computed<string>(() => 'flex-row items-center w-full min-w-0 px-2 py-[6px] border-b border-border hover:bg-accent text-[' + font.get().toString() + 'px]' + (active.get() ? ' bg-accent text-accent-foreground' : ''));
  return (
    <View hidden={!searchName.includes(query.get().trim().toLowerCase())}>
      <Button
        className={style.get()}
        variant="ghost"
        selected={active.get()}
        tooltip={clone(label)}
        aria-label={clone(label)}
        onPress={() => {
          selected.set(clone(name));
        }}
      >
        <Text className={'grow min-w-0 truncate text-[' + font.get().toString() + 'px]'}>
          {clone(label)}
        </Text>
      </Button>
    </View>
  );
}

function modelQuantization(model: ReadOnly<ModelExtension>): string {
  const parts = model.filename.split('-');
  return parts[parts.length - 1].split('.')[0].toUpperCase();
}

function modelLabel(model: ReadOnly<ModelExtension>): string {
  const quant = modelQuantization(model);
  return model.name.endsWith(' ' + quant) ? clone(model.name) : model.name + ' ' + quant;
}

export function ExtensionBrowser(selected: Signal<string>, font: Signal<int>, paths: Signal<string[]>): Node {
  const query = signal('');
  const rows: Node[] = [];

  for (const model of modelExtensions()) rows.push(ExtensionItem(clone(model.name), modelLabel(model), selected.handle(), font.handle(), query.handle()));
  return (
    <View className="flex-col size-full min-h-0">
      <InputGroup size="small" className="w-full min-w-0 rounded-none border border-transparent">
        <Input
          id="extension-search"
          value={query.get()}
          className={'px-2 text-[' + font.get().toString() + 'px]'}
          placeholder={message('Search extensions')}
          aria-label={accessibleName('Search extensions')}
          onChange={(event) => {
            query.set(clone(event.target.value));
          }}
        />
        <InputGroupAddon align="inline-end">
          <InputGroupButton
            icon="folder-open"
            tooltip={message('Open local model…')}
            accessibilityLabel={message('Open local model…')}
            onClick={() => {
              pickFilePaths((files) => {
                if (files.length === 0) return;
                const path = clone(files[0]);
                const catalog = modelExtensions().find((model) => join(modelFolder(model), model.filename) === path);

                if (catalog === undefined && !paths.get().includes(path)) paths.set([...paths.get(), clone(path)]);
                query.set('');
                selected.set(catalog === undefined ? path : clone(catalog.name));
              });
            }}
          />
        </InputGroupAddon>
      </InputGroup>
      <View className="grow min-h-0 overflow-y-auto flex-col">
        {...rows}
        {...paths.get().filter((path: ReadOnly<string>) => !modelExtensions().some((model: ReadOnly<ModelExtension>) => join(modelFolder(model), model.filename) === path)).map((path: ReadOnly<string>) => ExtensionItem(clone(path), basename(path), selected.handle(), font.handle(), query.handle()))}
      </View>
    </View>
  );
}

export function ModelExtensions(
  settings: Signal<StudioConfig>,
  paths: Signal<string[]>,
  ai: Signal<AiState>,
  commands: Sender<ModelRequest>,
  downloads: Sender<string>,
  selected: Signal<string>,
  font: Signal<int>,
  replies: Channel<string>
): Node {
  const pending = signal('');
  const error = signal('');
  const errorOwner = signal('');
  const rows: Node[] = [];

  for (const model of modelExtensions()) {
    const rowName = clone(model.name);
    rows.push(
      <View hidden={selected.get() !== rowName} className="flex-col gap-2">
        <Text className={'font-semibold text-[' + font.get().toString() + 'px]'}>
          {modelLabel(model)}
        </Text>
        {ModelRow(
          model,
          paths.handle(),
          pending.handle(),
          error.handle(),
          errorOwner.handle(),
          downloads.share(),
          font.handle(),
          settings.handle(),
          ai.handle(),
          commands.share()
        )}
      </View>
    );
  }

  return (
    <View className="flex-col size-full min-h-0 gap-2 p-3 overflow-y-auto">
      {...rows}
      <View hidden={!paths.get().includes(selected.get())}>
        {LocalModelDetail(
          selected.handle(),
          settings.handle(),
          ai.handle(),
          commands.share(),
          error.handle(),
          errorOwner.handle(),
          font.handle()
        )}
      </View>
      <View hidden={error.get() === '' || selected.get() !== errorOwner.get()}>
        <Text role="status" className={'text-destructive text-[' + font.get().toString() + 'px]'}>
          {localizeError(error.get())}
        </Text>
      </View>
      <Ticker
        running={pending.get() !== ''}
        interval={200}
        onTick={(_frame) => {
          match (replies.tryReceive()) {
            Ok(result) => {
              errorOwner.set(pending.get());
              pending.set('');
              error.set(result);
              paths.set(availableModelPaths(settings.get()));
            },
            Err(reason) => {
              if (reason.code !== 'empty') {
                errorOwner.set(pending.get());
                pending.set('');
                error.set(reason.code);
              }
            }
          }
        }}
      />
    </View>
  );
}

function LocalModelDetail(
  path: Signal<string>,
  settings: Signal<StudioConfig>,
  ai: Signal<AiState>,
  commands: Sender<ModelRequest>,
  error: Signal<string>,
  errorOwner: Signal<string>,
  font: Signal<int>
): Node {
  const current = computed<boolean>(() => settings.get().aiModelPath === path.get());
  return (
    <View className="flex-col items-start gap-2">
      <Text className={'text-[' + font.get().toString() + 'px]'}>
        {path.get()}
      </Text>
      <Button
        disabled={current.get() || !['ready', 'stopped'].includes(ai.get().phase)}
        onPress={() => {
          errorOwner.set(path.get());
          activateModel(path.get(), settings, ai, commands, error);
        }}
      >
        <Text className={'text-[' + font.get().toString() + 'px]'}>
          {current.get() ? translate('Active model') : translate('Use model')}
        </Text>
      </Button>
    </View>
  );
}

function activateModel(path: ReadOnly<string>, settings: Signal<StudioConfig>, ai: Signal<AiState>, commands: ReadOnly<Sender<ModelRequest>>, error: Signal<string>): void {
  if (path === '' || path === settings.get().aiModelPath || !['ready', 'stopped'].includes(ai.get().phase)) return;

  if (!path.toLowerCase().endsWith('.gguf')) {
    error.set('Choose a GGUF model file.');
    return;
  }

  try {
    const file = unwrap(configPath());
    let config = defaultConfig();
    let expected: string | undefined = undefined;

    match (readTextFile(file, 1024 * 1024)) {
      Ok(text) => {
        config = unwrap(parseConfig(text));
        expected = text;
      },
      Err(reason) => {
        if (reason.code !== 'not-found') {
          error.set(clone(reason.message));
          return;
        }
      }
    }

    const tokenizer = modelTokenizer(config, path);

    if (config.aiModelPath !== '' && !modelExtensions().some((model: ReadOnly<ModelExtension>) => join(modelFolder(model), model.filename) === config.aiModelPath)) {
      config.priorLocalModel = clone(config.aiModelPath);
      config.priorLocalTokenizer = clone(config.aiTokenizerPath);
    }

    if (!unwrap(stat(path)).isFile || !unwrap(stat(tokenizer)).isFile) {
      error.set('Model files are unavailable.');
      return;
    }

    const next = ai.get();

    if (next.phase === 'ready') {
      match (commands.trySend({
        kind: 'shutdown', id: '', prompt: '', maxTokens: 0
      })) {
        Ok(_) => {},
        Err(reason) => {
          error.set(clone(reason.code));
          return;
        }
      }

      next.phase = 'stopping';
      ai.set(next);
    }

    config.aiModelPath = clone(path);
    config.aiTokenizerPath = tokenizer;
    settings.set(unwrap(saveConfig(file, formatConfig(config), expected)));
    error.set('');
  } catch (reason) {
    if (typeof reason === 'string') error.set(reason);
    else error.set(clone(reason.message));
  }

}

function ModelRow(
  model: ReadOnly<ModelExtension>,
  paths: Signal<string[]>,
  pending: Signal<string>,
  error: Signal<string>,
  errorOwner: Signal<string>,
  downloads: Sender<string>,
  font: Signal<int>,
  settings: Signal<StudioConfig>,
  ai: Signal<AiState>,
  commands: Sender<ModelRequest>
): Node {
  const name = clone(model.name);
  const file = join(modelFolder(model), model.filename);
  const activePath = computed<string>(() => paths.get().includes(file) ? clone(file) : '');
  const statusName = clone(name);
  const status = computed<string>(() => pending.get() === statusName ? translate('Downloading') : activePath.get() === '' ? translate('Download') : activePath.get() === settings.get().aiModelPath ? translate('Active model') : translate('Use model'));
  const size = model.filename.split('-')[2].toUpperCase();
  const quant = modelQuantization(model);
  const style = computed<string>(() => 'text-[' + font.get().toString() + 'px]');
  return (
    <View className="flex-col items-start gap-2 min-w-0">
      <Text role="note" className={style.get()}>
        {translate(clone(model.description))}
      </Text>
      <Text className={style.get()}>
        {size + ' · ' + quant + ' · GGUF'}
      </Text>
      <Text className={style.get()}>
        {translate('Model file') + '  ' + model.filename}
      </Text>
      <Button
        size="small"
        disabled={pending.get() !== '' || (activePath.get() !== '' && (settings.get().aiModelPath === activePath.get() || !['ready', 'stopped'].includes(ai.get().phase)))}
        onPress={() => {
          errorOwner.set(clone(name));

          if (activePath.get() !== '') {
            activateModel(activePath.get(), settings, ai, commands, error);
            return;
          }

          match (downloads.trySend(clone(name))) {
            Ok(_) => {
              pending.set(clone(name));
              error.set('');
            },
            Err(reason) => {
              error.set(reason.code);
            }
          }
        }}
      >
        <Text className={'text-[' + font.get().toString() + 'px]'}>
          {status.get()}
        </Text>
      </Button>
    </View>
  );
}
