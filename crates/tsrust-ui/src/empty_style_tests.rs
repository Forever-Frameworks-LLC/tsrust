use super::*;
use style::{Utility,Length,State};

#[test]
fn empty_style_classes_keep_independent_properties_and_theme_references(){
 let theme=theme::Theme::default();
 let classes="border-dashed border border-r-0 text-sm/relaxed tracking-tight max-w-sm mb-2 flex-1 font-heading text-balance";
 assert!(style::validate(classes).is_ok());let values=style::resolved(classes,&theme,"light");
 assert!(values.iter().any(|(_,v)|matches!(v,Utility::BorderDashed(true))));
 assert!(values.iter().any(|(_,v)|matches!(v,Utility::Tracking(n)if *n== -0.025)));
 assert!(values.iter().any(|(_,v)|matches!(v,Utility::Dimension(4,Length::Pixels(384.)))));
 assert!(values.iter().any(|(_,v)|matches!(v,Utility::Leading(1.625))));
 assert!(values.iter().any(|(_,v)|matches!(v,Utility::Margin(2,8.))));
 assert!(values.iter().any(|(_,v)|matches!(v,Utility::BasisZero)));
 assert!(values.iter().any(|(_,v)|matches!(v,Utility::Balance(true))));
 assert!(values.iter().any(|(_,value)|matches!(value,Utility::EdgeBorder(1,0.))));
 let css=style::css(classes,&theme,"light",1,"");assert!(css.contains("border-style:dashed"));assert!(!css.contains("border-right-style:solid"));assert!(css.contains("letter-spacing:-0.025em"));assert!(css.contains("text-wrap-style:balance"));assert!(css.contains("line-height:1.625"));
 assert_eq!(style::theme_paths("max-w-sm"),vec![vec!["typography","base"]]);assert_eq!(style::theme_paths("mb-2"),vec![vec!["spacing","sm"]]);assert_eq!(style::theme_paths("font-heading"),vec![vec!["typography","headingFamily"]]);
 for class in classes.split_whitespace(){assert!(!style::documentation(class).is_empty(),"{class}");}
 assert!(style::validate_inline("tracking-tight text-balance text-sm/relaxed").is_ok());assert!(style::validate_inline("mb-2").is_err());
 for class in ["tracking-arbitrary","leading-unknown","-mb-2","m-auto","max-w-made-up"]{assert!(style::validate(class).is_err(),"{class}");}
}

#[test]
fn empty_descendant_selectors_match_targets_without_styling_the_parent(){
 let mut theme=theme::Theme::default();theme.spacing.md=20.;
 let classes="[&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4";
 let parent=Node::view(vec![],classes.into()).unwrap();let svg=Node::svg(vec![],String::new(),"{}").unwrap();
 assert!(style::resolved_node(&parent,&[],&theme,"light").is_empty());let values=style::resolved_node(&svg,&[&parent],&theme,"light");assert!(values.iter().any(|(_,v)|matches!(v,Utility::Dimension(0,Length::Pixels(20.)))));assert!(values.iter().any(|(_,v)|matches!(v,Utility::PointerEvents(false))));
 let sized=Node::svg(vec![],"size-6".into(),"{}").unwrap();let values=style::resolved_node(&sized,&[&parent],&theme,"light");assert!(values.iter().any(|(_,v)|matches!(v,Utility::Dimension(0,Length::Pixels(24.)))));assert!(!values.iter().any(|(_,v)|matches!(v,Utility::Dimension(0,Length::Pixels(20.)))));
 let css=style::css(classes,&theme,"light",1,"");assert!(css.contains(":where(svg:not([class*='size-']))"));assert!(css.contains("width:var(--ui-spacing-md);height:var(--ui-spacing-md)"));
 let description=Node::text(vec![],"[&>a]:underline [&>a]:underline-offset-4 [&>a:hover]:text-primary".into()).unwrap();let link=navigation::link(vec![],String::new(),"/docs").unwrap();
 let rules=style::resolved_node(&link,&[&description],&theme,"light");assert!(rules.iter().any(|(_,v)|matches!(v,Utility::Underline(true))));assert!(rules.iter().any(|(state,v)|*state==State::Hover&&matches!(v,Utility::Foreground(_))));
 let intermediary=Node::text(vec![],String::new()).unwrap();assert!(style::resolved_node(&link,&[&description,&intermediary],&theme,"light").is_empty());
 for invalid in ["[&_button]:size-4","[&_svg]:absolute","[&>a]:p-4","[&_svg]:[&>a]:underline"]{assert!(style::validate(invalid).is_err(),"{invalid}");}
}

#[test]
fn empty_heading_family_is_optional_and_checked(){
 let default=theme::Theme::parse(theme::DEFAULT_JSON).unwrap();assert_eq!(default.typography.heading_family.index(),0);
 let mut value:serde_json::Value=serde_json::from_str(theme::DEFAULT_JSON).unwrap();
 for (family,index) in [("sans",0),("serif",1),("mono",2)]{value["typography"]["headingFamily"]=family.into();let theme=theme::Theme::parse(&value.to_string()).unwrap();assert_eq!(theme.typography.heading_family.index(),index);assert!(style::resolved("font-heading",&theme,"light").iter().any(|(_,v)|matches!(v,Utility::FontFamily(found)if *found==index)));}
 for invalid in [serde_json::Value::Null,42.into(),"Comic Sans".into()]{value["typography"]["headingFamily"]=invalid;assert!(theme::Theme::parse(&value.to_string()).is_err());}
}
