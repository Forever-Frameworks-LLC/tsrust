interface TextProps {
  children: string[];
}

function Text(props: TextProps): string {
  return props.children.join('');
}

interface ViewProps {
  children: string[];
}

function View(props: ViewProps): string {
  return props.children.join('|');
}

function main() {
  const label = 'café';
  console.log(<View><Text>Hello {label}</Text><Text>🦀 &amp; friends</Text></View>);
}
