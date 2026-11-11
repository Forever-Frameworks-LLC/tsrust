import { View, Text, Button, Input, Node, signal, run, t, tWith, translate, translateWith, setTranslations, setLocale, locale, DatePicker, Select } from 'std:ui';
import en from './locales/en.json';
import fr from './locales/fr.json';

function App(): Node {
  const count = signal(2);
  const current = signal(locale());
  return (
    <View className="flex-col size-full items-center justify-center gap-3">
      <Text className="text-2xl font-semibold">{t('title')}</Text>
      <Text>{tWith('greeting', { name: 'Ada' })}</Text>
      <Text>{translateWith('messages', { count: count.get() })}</Text>
      <Input placeholder={translate('search')} />
      <Select items={['en', 'fr']} value={current.get()} placeholder={t('language')} onChange={(value) => { current.set(clone(value)); setLocale(value); }} className="w-48" />
      <DatePicker value="2026-11-09" className="w-48" />
      <Button onPress={() => { count.set(count.get() + 1); }}>+1</Button>
    </View>
  );
}

function main() {
  setTranslations('en', en);
  setTranslations('fr', fr);
  setLocale('en');
  unwrap(run(App(), 'i18n', {
    width: 520, height: 520
  }));
}
