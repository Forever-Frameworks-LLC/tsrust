mod support;
#[test]
fn deferred_values_cross_exception_transport_without_losing_owners(){
 let source=r#"import {strictEqual,trackedResource,TrackedResource,resourceId,releaseCount} from 'std:testing';
 import {Channel} from 'std:futures';
 type Item={text:string;};
 function checkResources(){
  let observed=0;
  for(let index=0;index<2;index++){
   let item:TrackedResource;
   let record:Item;
   let scalar:int;
   try{
    if(index===1){throw 'stop';}
    item=trackedResource();
    record={text:'kept'};
    scalar=42;
   }catch(error){break;}
   observed=resourceId(item);
   strictEqual(releaseCount(observed),0);
   strictEqual(record.text,'kept');
   strictEqual(scalar,42);
  }
  strictEqual(releaseCount(observed),1);
 }
 async function main():Task<void>{
  checkResources();
  const queue=new Channel<Item>(1);
  const initial:Item={text:'awaited'};
  await queue.send(initial);
  queue.close();
  let count=0;
  for(let index=0;index<2;index++){
   let item:Item;
   try{item=await queue.receive();}catch(error){break;}
   strictEqual(item.text,'awaited');
   count++;
  }
  strictEqual(count,1);
  console.log('exception initialization accepted');
 }"#;
 let directory=support::TempDirectory::new();let source=support::format_round_trip(source);
 let result=support::run(support::cli().arg("run").arg(directory.source(&source)),180);support::success(&result);assert_eq!(result.stdout,b"exception initialization accepted\n");
}

#[test]
fn uninitialized_reads_still_reject_before_emission(){
 let source="function read(flag:boolean):int{let value:int;try{if(flag){value=1;}else{throw 'missing';}}catch(error){}return value;}function main(){}";
 assert!(tsrust_compiler::check(&support::format_round_trip(source)).is_err());
}
