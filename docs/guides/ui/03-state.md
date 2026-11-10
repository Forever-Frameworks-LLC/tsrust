# State and Reactivity

State lives in signals. Reading `signal.get()` inside TSRX markup makes that part of
the tree update when the signal changes; nothing else re-renders.

## Signals

```tsx live Temperature
function Temperature(): Node {
  const celsius = signal(21.0);
  return (
    <View className="flex-col gap-3 w-80">
      <Slider min={-20} max={40} value={celsius.get()} onChange={(value) => { celsius.set(value); }} />
      <Text>{Math.round(celsius.get())} °C is {Math.round(celsius.get() * 9 / 5 + 32)} °F</Text>
    </View>
  );
}
```

`computed(() => ...)` names a derived value; it re-evaluates when read.

## Controlled and uncontrolled

A form control is controlled when you pass its value (`checked`, `value`,
`selected`, `open`) and update it from the change event. Leave the value out
and the control keeps its own state; `defaultChecked`, `defaultValue` or
`defaultOpen` set where it starts.

```tsx live ControlledSwitch
function ControlledSwitch(): Node {
  const on = signal(true);
  return (
    <View className="flex-col gap-3">
      <Switch checked={on.get()} onChange={(next) => { on.set(next); }}>Controlled: {on.get() ? 'on' : 'off'}</Switch>
      <Switch defaultChecked={true}>Uncontrolled</Switch>
    </View>
  );
}
```

## Conditional content

A ternary that reads a signal swaps subtrees; `hidden` keeps a subtree but
hides it.

```tsx live Conditional
function Conditional(): Node {
  const signedIn = signal(false);
  return (
    <View className="flex-col gap-3">
      <Button onPress={() => { signedIn.set(!signedIn.get()); }}>{signedIn.get() ? 'Sign out' : 'Sign in'}</Button>
      {signedIn.get() ? <Alert variant="success"><AlertTitle>Welcome back</AlertTitle></Alert> : <Text variant="muted">You are signed out.</Text>}
    </View>
  );
}
```

## Lists

Spread an array of nodes into children with `{...items.map(...)}`. When the
expression reads a signal, the list updates; rows whose content did not
change keep their state. Array callbacks borrow each element, so `clone()`
copies a string into the row it builds.

```tsx live TodoList
function TodoList(): Node {
  const count = signal(3);
  const tasks: string[] = ['Write docs', 'Review the PR', 'Ship 0.2.0', 'Plan GPUI showcase', 'Celebrate'];
  return (
    <View className="flex-col gap-2 w-80">
      {...tasks.filter((task: string, index: int) => index < count.get()).map((task: string) => <Checkbox>{clone(task)}</Checkbox>)}
      <Button outline={true} disabled={count.get() >= 5} onPress={() => { count.set(count.get() + 1); }}>Add task</Button>
    </View>
  );
}
```
