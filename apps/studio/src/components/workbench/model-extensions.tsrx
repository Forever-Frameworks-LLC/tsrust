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
  Select,
  SelectItem,
  computed,
  translate,
  signal
} from 'std:ui';
import { message, localizeError } from '../../lib/localization.tsr';
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

export function ModelExtensions(
  settings: Signal<StudioConfig>,
  paths: Signal<string[]>,
  ai: Signal<AiState>,
  commands: Sender<ModelRequest>,
  downloads: Sender<string>,
  replies: Channel<string>
): Node {
  const pending = signal('');
  const error = signal('');
  const rows: Node[] = [];

  for (const model of modelExtensions()) {
    rows.push(ModelRow(model, paths.handle(), pending.handle(), error.handle(), downloads.share()));
  }

  return (
    <View className="flex-col size-full min-h-0 gap-2 p-2 overflow-y-auto">
      <Select
        value={settings.get().aiModelPath}
        placeholder={message('Select model')}
        accessibilityLabel={message('Select model')}
        disabled={!['ready', 'stopped'].includes(ai.get().phase)}
        onChange={(path) => {
          if (path === '' || path === settings.get().aiModelPath || !['ready', 'stopped'].includes(ai.get().phase)) return;

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
        }}
      >
        {...paths.get().map((path: ReadOnly<string>) => <SelectItem value={clone(path)} label={basename(path)} />
        )}
      </Select>
      {...rows}
      <View hidden={error.get() === ''}>
        <Text className="text-destructive">
          {localizeError(error.get())}
        </Text>
      </View>
      <Ticker
        running={pending.get() !== ''}
        interval={200}
        onTick={(_frame) => {
          match (replies.tryReceive()) {
            Ok(result) => {
              pending.set('');
              error.set(result);
              paths.set(availableModelPaths(settings.get()));
            },
            Err(reason) => {
              if (reason.code !== 'empty') {
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

function ModelRow(model: ReadOnly<ModelExtension>, paths: Signal<string[]>, pending: Signal<string>, error: Signal<string>, downloads: Sender<string>): Node {
  const name = clone(model.name);
  const file = join(modelFolder(model), model.filename);
  const installed = computed<boolean>(() => paths.get().includes(file));
  const statusName = clone(name);
  const status = computed<string>(() => pending.get() === statusName ? translate('Downloading') : installed.get() ? translate('Installed') : translate('Download'));
  return (
    <View className="flex-row items-center gap-2 min-w-0 px-2">
      <Text className="grow min-w-0 truncate">
        {model.filename.slice(0, -5)}
      </Text>
      <Button
        variant="ghost"
        size="small"
        disabled={pending.get() !== '' || installed.get()}
        onPress={() => {
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
        {status.get()}
      </Button>
    </View>
  );
}
