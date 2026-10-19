mod support;
#[test]
fn formatted_inline_semantics_and_exact_clipboard_snapshot_have_native_support(){
 let source=r#"import {View,Text,Link,CopyButton,Node,location,document} from 'std:ui';import {strictEqual} from 'std:testing';
 function Page():Node{const path=location();return <View><Text as='p'>Read <Text as='strong'>bold <Text as='em'>café 文</Text></Text> and <Link href='/docs'>docs</Link>. <Text as='del'>old</Text>{1}{2.5}</Text><CopyButton value={'café 文\r\n  '} className='rounded border'/></View>;}
 function main(){const html=document(Page(),'Inline');strictEqual(html.includes('<strong'),true);strictEqual(html.includes('<em'),true);strictEqual(html.includes('<del'),true);strictEqual(html.includes('12.5'),true);strictEqual(html.includes('href=\"/docs\"'),true);strictEqual(html.includes('data-copy-state=\"idle\"'),true);strictEqual(html.includes('>Copy</button>'),true);console.log('Inline content passed');}"#;
 let directory=support::TempDirectory::new();let entry=directory.0.join("main.tsrx");std::fs::write(&entry,support::format_round_trip(source)).unwrap();std::fs::write(directory.0.join("tsrust.config.json"),serde_json::json!({"ui":tsrust_compiler::ui_project::Config::default()}).to_string()).unwrap();let output=support::run(support::cli().arg("run").arg(entry),120);support::success(&output);assert_eq!(output.stdout,b"Inline content passed\n");
}
#[test]fn code_lexemes_reconstruct_unicode_crlf_and_unknown_languages_exactly(){
 for (source,language) in [("// café 文\r\nconst message = '😀';\r\n  ","tsrust"),("opaque < 文 >\r\n","unknown")] {
  let tokens=tsrust_compiler::documentation::highlight(source,language).unwrap();assert_eq!(tokens.iter().map(|token|token.text.as_str()).collect::<String>(),source);for token in tokens {assert_eq!(token.text,source[token.source.start..token.source.end]);}
 }
}
