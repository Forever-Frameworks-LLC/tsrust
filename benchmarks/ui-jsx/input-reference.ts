// Independent application-state oracle; no UI renderer or compiler helpers.
declare const console:{log(value:string):void};
let name='Ada',notes='First',locked=false,edits=0,uncontrolled='Initial',last='';
const observations:unknown[]=[];
function observe(){observations.push({name,notes,derived:name+'!',locked,edits,uncontrolled,last});}
function change(value:string){if(!locked&&value!==name){name=value.replace(/[\r\n]/g,'');edits++;last='name:name';}}
observe();change('Grace');observe();change('Gréace');observe();
// Preedit has no committed change; final composition commits exactly once.
observe();change('日本🙂');observe();change('日本🙂');observe();
notes='Line1\nLine2';observe();locked=true;observe();change('ignored');observe();
name='Reset';notes='One\nTwo';observe();uncontrolled='Changed';observe();
console.log(JSON.stringify({observations,errors:['Value rejected','Handler rejected','DOM control value contains an unpaired UTF-16 surrogate']}));
