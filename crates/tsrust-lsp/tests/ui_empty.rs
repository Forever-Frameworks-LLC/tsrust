mod support;
#[test]
fn empty_utilities_and_heading_theme_fields_have_current_documentation(){
 let source="import {View,Text,ThemeTypography} from 'std:ui';function main(){const typography:ThemeTypography={base:16,xl:24,headingFamily:'serif'};const node=<View className=\"border border-dashed max-w-sm mb-2 [&_svg]:size-4\"><Text className=\"text-sm/relaxed tracking-tight text-balance font-heading\">Create your first project</Text></View>;}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");session.settle();
 for value in ["border-dashed","max-w-sm","mb-2","text-sm/relaxed","tracking-tight","text-balance","font-heading","headingFamily"]{let hover=session.hover("main.tsrx",source,source.find(value).unwrap()+2).unwrap_or_default();assert!(!hover.is_empty(),"{value}");}
 let counters=session.request("tsrust/counters",serde_json::Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{counters}");}
}

#[test]
fn inline_heading_theme_config_offers_checked_values_and_unsaved_errors(){
 let mut theme=serde_json::to_value(tsrust_compiler::ui_project::theme::Theme::default()).unwrap();theme["typography"]["headingFamily"]="serif".into();
 let config=serde_json::json!({"schemaVersion":1,"name":"empty-theme","build":{"entry":"src/main.tsrx","output":"target/app"},"ui":{"target":{"native":true,"web":true,"embedded":false},"theme":theme,"components":"src/components/ui","lock":"ui-components.lock.json"}}).to_string();
 let source="import {Text,withTheme} from 'std:ui';import config from '../tsrust.config.json';function main(){const node=withTheme(<Text className=\"font-heading\">Title</Text>,config.ui.theme);}";
 let mut session=support::session::Session::start(&[("tsrust.config.json",&config),("src/main.tsrx",source)]);session.open("src/main.tsrx",source);session.open("tsrust.config.json",&config);session.settle();
 let key=config.find("headingFamily").unwrap()+2;let hover=session.hover("tsrust.config.json",&config,key).unwrap_or_default();assert!(hover.contains("sans")&&hover.contains("Optional"),"{hover}");assert!(session.completion("tsrust.config.json",&config,key).unwrap().contains(&"headingFamily".into()));
 let value=config.find("\"serif\"").unwrap()+1;let offered=session.completion("tsrust.config.json",&config,value).unwrap();for family in ["sans","serif","mono"]{assert!(offered.contains(&family.into()),"{offered:?}");}assert!(!offered.contains(&"system-ui".into()));
 let bad=config.replacen("\"serif\"","\"unknown\"",1);session.change("tsrust.config.json",&bad);let mut diagnostics=session.wait_for_diagnostics("tsrust.config.json");for _ in 0..3{if diagnostics.to_string().contains("unknown"){break;}diagnostics=session.wait_for_diagnostics("tsrust.config.json");}assert!(diagnostics.to_string().contains("unknown"),"{diagnostics}");session.change("tsrust.config.json",&config);session.settle();
}
