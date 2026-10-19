// Independent source contract, including which ancestor/descendant conditions match.
// Does not import the runtime's selector parser or geometry.
declare const console:{log(value:string):void};
const expected=[
 {size:'sm',padding:['12px','12px','0px','12px'],bodyPadding:['16px','12px','16px','12px'],footerTop:'24px',minHeight:'32px'},
 {size:'default',padding:['16px','16px','16px','16px'],bodyPadding:['16px','16px','16px','16px'],footerTop:null,minHeight:'0px'},
];
console.log(JSON.stringify(expected));
