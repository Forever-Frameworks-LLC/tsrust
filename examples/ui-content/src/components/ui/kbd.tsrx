/*
MIT License

Copyright (c) 2023 shadcn

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
import { View, ViewProps, Node } from 'std:ui';

export type KbdProps = ViewProps;
/** Displays keyboard text. It does not install a keyboard shortcut. */
export function Kbd(props: KbdProps): Node {
  const classes = 'pointer-events-none inline-flex flex-row items-center justify-center select-none bg-muted text-muted-foreground in-data-[slot=tooltip-content]:bg-background/20 in-data-[slot=tooltip-content]:text-background dark:in-data-[slot=tooltip-content]:bg-background/10 h-5 w-fit min-w-5 gap-1 rounded-sm px-1 font-sans text-xs font-medium ' + (props.className ?? '');
  return (<View {...props} as="kbd" data-slot="kbd" className={classes}/>);
}
/** Groups keycaps and separators in source order. */
export function KbdGroup(props: KbdProps): Node {
  const classes = 'inline-flex flex-row items-center gap-1 ' + (props.className ?? '');
  return (<View {...props} as="kbd" data-slot="kbd-group" className={classes}/>);
}
