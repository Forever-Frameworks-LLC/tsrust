function receiver():number[]{console.log("receiver");return [1];}
function scalar0():number{console.log("scalar0");return 2;}
function array0():number[]{console.log("array0");return [3,4];}
function scalar1():number{console.log("scalar1");return 5;}
function empty():number[]{console.log("empty");return [];}
function array1():number[]{console.log("array1");return [6];}
function singleton():number[]{console.log("singleton");return [9];}
function main(){const output=receiver().concat(scalar0(),array0(),scalar1(),empty(),array1());for(const value of output){console.log(value);}const zero=empty().concat();console.log(zero.length);const one=singleton().concat([],scalar0());for(const value of one){console.log(value);}}
