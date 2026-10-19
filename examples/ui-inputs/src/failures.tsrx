// Dedicated lifecycle witness; failure policy belongs to this application.
import { Input, Node, signal, run } from 'std:ui';

function checkedValue(value: ReadOnly<string>): string {
  if (value === 'bad') {
    throw 'Value rejected';
  }

  return clone(value);
}

function GetterFailure(): Node {
  const value = signal('fine');
  return (<Input aria-label="Throwing getter" value={checkedValue(value.get())} onChange={(event)=>{value.set(event.target.value);}}/>);
}

function HandlerFailure(): Node {
  return (<Input aria-label="Throwing handler" defaultValue="fine" onChange={(event)=>{throw 'Handler rejected';}}/>);
}

function main() {
  unwrap(run(GetterFailure(), 'Getter failure'));
  unwrap(run(HandlerFailure(), 'Handler failure'));
  unwrap(run(<Input aria-label="Invalid UTF16" defaultValue="fine"/>, 'Unicode boundary'));
}
