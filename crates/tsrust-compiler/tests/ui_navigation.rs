mod support;
#[test]
fn formatted_application_location_links_and_native_history_preserve_owners(){
 let source=r#"import {View,Text,Link,Node,location,navigate,back,forward,document} from 'std:ui';import {strictEqual} from 'std:testing';
 function App():Node{const path=location();navigate('/docs');strictEqual(path.get(),'/docs');back();strictEqual(path.get(),'/');forward();strictEqual(path.get(),'/docs');return <View><Link href='/docs/getting-started'>Read docs</Link><Text>{path.get()}</Text><View hidden={path.get()!=='/docs'}><Text>Article</Text></View></View>;}
 function main(){const app=App();const html=document(app,'Site');strictEqual(html.includes('href=\"/docs/getting-started\"'),true);strictEqual(html.includes('Read docs'),true);console.log('Navigation passed');}"#;
 support::execute(source,"Navigation passed\n",None);
}
