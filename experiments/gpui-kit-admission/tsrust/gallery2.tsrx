import {
  View, Text, Button, Node, signal, run, setTheme,
  Markdown, Html, Image, TitleBar, StatusBar, StatusBarLeft, StatusBarRight, Toolbar, Form, Field, ScrollArea, Message,
  Chart, List, Tree, TreeItem, Calendar, TimeField, Combobox, Command, CommandItem, Carousel, CarouselSlide, CodeEditor,
  ToggleGroup, Toggle, ButtonGroup, GroupButton, Input, Switch
} from 'std:ui';

function Gallery(): Node {
  const dark = signal(false);
  const picked = signal(1);
  const day = signal('2026-11-09');
  const time = signal('09:30:00');
  const framework = signal('Svelte');
  const code = signal('fn main() {\n    println!("hello");\n}\n');
  const last = signal('none');
  const bold = signal(true);
  return (
    <View className="flex-col size-full">
      <Toolbar><Button onPress={() => { last.set('new'); }}>New</Button><Switch label="Dark" checked={dark.get()} onChange={(on) => { dark.set(on); setTheme(on ? 'dark' : 'light'); }} /></Toolbar>
      <ScrollArea className="grow">
        <View className="flex-col gap-4 p-6">
          <Markdown content={'## Markdown\n\nSome **bold** text, a [link](https://gpui-kit.com) and a list:\n\n- one\n- two'} />
          <Html content="<em>HTML</em> renders <strong>inline</strong>." />
          <View className="flex-row gap-4 h-48">
            <Chart kind="bar" labels={['Jan', 'Feb', 'Mar', 'Apr']} values={[100, 150, 120, 180]} className="w-64 h-48" />
            <Chart kind="line" labels={['Jan', 'Feb', 'Mar', 'Apr']} values={[3, 7, 4, 9]} className="w-64 h-48" />
            <Chart kind="pie" labels={['A', 'B', 'C']} values={[50, 30, 20]} className="w-64 h-48" />
          </View>
          <View className="flex-row gap-4">
            <List items={['Inbox', 'Drafts', 'Sent', 'Archive', 'Spam', 'Trash']} selected={picked.get()} onSelect={(index) => { picked.set(index); }} className="w-48 h-48" />
            <Tree onSelect={(id) => { last.set(clone(id)); }} className="w-56 h-48">
              <TreeItem id="src" label="src" expanded={true}><TreeItem id="src/main.tsr" label="main.tsr" /><TreeItem id="src/ui.tsrx" label="ui.tsrx" /></TreeItem>
              <TreeItem id="README.md" label="README.md" />
            </Tree>
            <Calendar value={day.get()} onChange={(date) => { day.set(clone(date)); }} />
          </View>
          <Text>Day {day} · time {time} · framework {framework} · last {last}</Text>
          <View className="flex-row gap-4 items-center">
            <TimeField value={time.get()} onChange={(value) => { time.set(clone(value)); }} />
            <Combobox items={['React', 'Svelte', 'Solid', 'Vue']} value={framework.get()} placeholder="Framework" onChange={(value) => { framework.set(clone(value)); }} className="w-56" />
            <ToggleGroup><Toggle label="B" checked={bold.get()} onChange={(on) => { bold.set(on); }} /><Toggle label="I" /></ToggleGroup>
            <ButtonGroup><GroupButton label="Day" onClick={() => { last.set('day'); }} /><GroupButton label="Week" onClick={() => { last.set('week'); }} /></ButtonGroup>
          </View>
          <Form columns={2}><Field label="Name" required={true}><Input placeholder="Ada" /></Field><Field label="Email" description="Never shared."><Input placeholder="ada@example.com" /></Field></Form>
          <Message author="Alice" time="10:24"><Text>Can you review this draft?</Text></Message>
          <Message author="You" time="10:25" align="end" variant="primary"><Text>On it.</Text></Message>
          <Carousel className="h-32"><CarouselSlide><Text>Slide one</Text></CarouselSlide><CarouselSlide><Text>Slide two</Text></CarouselSlide></Carousel>
          <Command placeholder="Type a command" className="h-48"><CommandItem label="New file" onSelect={() => { last.set('new file'); }} /><CommandItem label="Settings" onSelect={() => { last.set('settings'); }} /></Command>
          <Text>Last {last}</Text>
          <CodeEditor language="rust" value={code.get()} onChange={(text) => { code.set(clone(text)); }} className="h-40" />
          <Image src="https://gpui-kit.com/favicon.ico" alt="logo" className="size-12" />
        </View>
      </ScrollArea>
      <StatusBar><StatusBarLeft><Text>Ready</Text></StatusBarLeft><StatusBarRight><Text>UTF-8</Text></StatusBarRight></StatusBar>
    </View>
  );
}

function main() {
  unwrap(run(Gallery(), 'Tsrust GPUI Kit gallery 2', {
    width: 1100, height: 820
  }));
}
