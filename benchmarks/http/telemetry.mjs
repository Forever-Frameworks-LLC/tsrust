import {execFileSync} from 'node:child_process';
export function processSnapshot(pid){
 const text=execFileSync('ps',['-p',String(pid),'-o','time=','-o','rss='],{encoding:'utf8'}).trim();
 return parseSnapshot(text);
}
export function parseSnapshot(text){
 const match=/^(?:(\d+)-)?(?:(\d+):)?(\d+):(\d+(?:\.\d+)?)\s+(\d+)$/.exec(text);
 if(!match)throw Error('Invalid process CPU/RSS sample: '+text);
 return {cpuMs:((+(match[1]??0)*24+ +(match[2]??0))*3600+ +match[3]*60+ +match[4])*1000,rssBytes:+match[5]*1024};
}
