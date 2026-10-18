declare const console: { log(value: string): void };
namespace JSX {
  export type Element = string;
  export interface ElementChildrenAttribute { children: {}; }
}
function element(component: (props: any) => string, props: any, ...children: string[]): string {
  return component({...props, children});
}
interface TextProps { children: string | string[]; }
function Text(props: TextProps): string { return typeof props.children === 'string' ? props.children : props.children.join(''); }
interface ViewProps { children: string | string[]; }
function View(props: ViewProps): string { return typeof props.children === 'string' ? props.children : props.children.join('|'); }
const label = 'café';
console.log(<View><Text>Hello {label}</Text><Text>🦀 &amp; friends</Text></View>);
