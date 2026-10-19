import { View, Text, Node, Child } from 'std:ui';
import { DocPage, DocApi, DocCatalog } from '../../../target/docs-content/catalog';

function componentName(name: string): string {
  let result = '';
  for (const part of name.split('-')) {
    result += part.slice(0, 1).toUpperCase() + part.slice(1);
  }

  return result;
}

function belongs(page: ReadOnly<DocPage>, declaration: ReadOnly<DocApi>): boolean {
  const component = clone(page.metadata.component);
  if (component !== null) {
    const name = componentName(component);
    if (declaration.source.endsWith('/' + component + '.tsrx') || declaration.name === name || declaration.name === name + 'Props') {
      return true;
    }
  }

  for (const reference of page.metadata.api) {
    if (declaration.name === reference.receiver + '.' + reference.name) {
      return true;
    }
  }

  return false;
}

export function searchablePage(page: ReadOnly<DocPage>, catalog: ReadOnly<DocCatalog>): string {
  let text = page.document.title + ' ' + page.markdown;
  for (const declaration of catalog.api) {
    if (!belongs(page, declaration)) {
      continue;
    }

    text += ' ' + declaration.name + ' ' + declaration.signature + ' ' + declaration.documentation;
    for (const field of declaration.fields) {
      text += ' ' + field.name + ' ' + field.signature + ' ' + field.documentation;
    }
  }

  for (const id of page.metadata.examples) {
    for (const example of catalog.examples) {
      if (example.id === id) {
        text += ' ' + example.code;
      }
    }
  }

  return text.toLowerCase();
}

export function ApiReference(page: ReadOnly<DocPage>, catalog: ReadOnly<DocCatalog>): Node {
  let entries: Child[] = [];
  for (const declaration of catalog.api) {
    if (!belongs(page, declaration)) {
      continue;
    }

    let fields: Child[] = [];
    for (const field of declaration.fields) {
      fields.push(<View className="border-t py-4 gap-2"><Text as="h4" className="font-mono text-sm font-semibold">{clone(field.name)}{field.optional ? '?' : ''}</Text><Text as="code" className="font-mono text-xs text-muted-foreground">{clone(field.signature)}</Text><Text className="leading-normal text-sm text-muted-foreground">{clone(field.documentation)}</Text></View>);
    }

    entries.push(<View className="border-t pt-6 gap-4"><Text as="h3" className="font-semibold text-xl">{clone(declaration.name)}</Text><Text as="code" className="font-mono text-sm">{clone(declaration.signature)}</Text><Text className="leading-normal text-muted-foreground">{clone(declaration.documentation)}</Text><View children={fields}/></View>);
  }

  return (<View className="gap-6" children={entries}/>);
}
