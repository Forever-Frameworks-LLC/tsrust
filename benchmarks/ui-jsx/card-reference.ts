// Independent expected style values from the pinned Nova rules and the fixture tokens.
// Does not import any Tsrust utility, selector, layout or color implementation.
declare const console:{log(value:string):void};
const expected={
 default:{paddingTop:'16px',paddingBottom:'0px',gap:'16px',radius:'14px',headerPadding:'16px',titleSize:'16px',titleLeading:'22px',titleWeight:'500',footerPadding:'16px',footerTop:'1px'},
 small:{paddingTop:'12px',paddingBottom:'12px',gap:'12px',radius:'14px',headerPadding:'12px',titleSize:'14px',titleLeading:'19.25px',titleWeight:'500'},
 custom:{paddingTop:'24px',paddingBottom:'0px',gap:'24px',radius:'0px',headerPadding:'24px',headerBottom:'24px',footerPadding:'24px'},
 missing:'0px',
};
console.log(JSON.stringify(expected));
