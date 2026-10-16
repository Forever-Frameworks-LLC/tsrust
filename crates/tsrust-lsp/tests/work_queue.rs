mod support;
#[test]
fn bounded_queue_example_has_typed_completion_and_help(){
 use support::session::Session;
 let source=include_str!("../../../examples/work-queue/main.tsr");
 let mut session=Session::start(&[]);session.open("main.tsr",source);session.settle();
 let send=source.find("sender.send").unwrap()+7;
 let hover=session.hover("main.tsr",source,send).unwrap_or_default();assert!(hover.contains("Work")&&hover.contains("send"),"{hover}");
 let before=source.find("queue.receive").unwrap()+6;
 let completion=session.completion("main.tsr",source,before).unwrap();
 for member in ["receive","send","close","sender"]{assert!(completion.iter().any(|label|label==member),"missing {member}: {completion:?}");}
}
