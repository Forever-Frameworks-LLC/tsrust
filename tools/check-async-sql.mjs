// Explicit integration runner: creates and removes only its own disposable containers.
import { spawnSync } from 'node:child_process';
import { randomUUID } from 'node:crypto';
import { setTimeout } from 'node:timers/promises';
if (!process.argv.includes('--docker')) throw new Error('Opt-in only: node tools/check-async-sql.mjs --docker');
function command(program,args,options={}) {
 const result=spawnSync(program,args,{encoding:'utf8',...options});
 if(result.error||result.status!==0)throw new Error(`${program} failed: ${result.error??result.stderr??result.status}`);
 return result.stdout?.trim();
}
const resume=process.argv.find(value=>value.startsWith('--resume-from='))?.slice('--resume-from='.length);
if(resume && !['postgres','mysql'].includes(resume))throw new Error('Resume must name postgres or mysql; retain prior passing evidence.');
const suffix=randomUUID().slice(0,8),containers=[];
function cleanup(){for(const name of containers.splice(0))spawnSync('docker',['rm','-f','-v',name],{stdio:'ignore'});}
for(const signal of ['SIGINT','SIGTERM'])process.on(signal,()=>{cleanup();process.exit(130);});
try {
 const definitions=[['postgres','postgres:18',5432,['POSTGRES_USER=tsrust','POSTGRES_PASSWORD=tsrust-local-test','POSTGRES_DB=tsrust'],['pg_isready','-U','tsrust']],['mysql','mysql:8.4',3306,['MYSQL_ROOT_PASSWORD=tsrust-local-root','MYSQL_USER=tsrust','MYSQL_PASSWORD=tsrust-local-test','MYSQL_DATABASE=tsrust'],['mysqladmin','ping','-h','127.0.0.1','-utsrust','-ptsrust-local-test']]];
 const env={...process.env,CARGO_INCREMENTAL:'0',CARGO_BUILD_JOBS:'1',CARGO_TARGET_DIR:process.env.CARGO_TARGET_DIR??'/private/tmp/tsrust-gate100-editor-target',TSRUST_CACHE_DIR:process.env.TSRUST_CACHE_DIR??'/private/tmp/tsrust-gate100-editor-target/sql-acceptance-cache'};
 delete env.TSRUST_TEST_SQL_RESUME;
 if(resume)env.TSRUST_TEST_SQL_RESUME=resume;
 for(const [driver,image,port,settings,probe]of definitions){
  const name=`tsrust-sql-${driver}-${suffix}`;containers.push(name);
  command('docker',['run','--detach','--rm','--name',name,'--publish',`127.0.0.1::${port}`,...settings.flatMap(value=>['--env',value]),image]);
  let ready=false;
  for(let attempt=0;attempt<120;attempt++){if(spawnSync('docker',['exec',name,...probe],{stdio:'ignore'}).status===0){ready=true;break;}await setTimeout(1000);}
  if(!ready)throw new Error(`${driver} did not become ready`);
  const address=command('docker',['port',name,`${port}/tcp`]);
  env[driver==='postgres'?'TSRUST_TEST_POSTGRES_URL':'TSRUST_TEST_MYSQL_URL']=`${driver}://tsrust:tsrust-local-test@${address}/tsrust`;
 }
 if(!resume)command('cargo',['test','-p','tsrust-compiler','--lib','stdlib::sql::tests','--','--ignored','--nocapture'],{env,stdio:'inherit'});
 command('cargo',['test','-p','tsrust-compiler','--test','async_sql','--','--ignored','--nocapture'],{env,stdio:'inherit'});
 command('cargo',['test','-p','tsrust-compiler','--test','http','sql_and_https_features_share_one_generated_program_without_provider_ambiguity','--','--ignored','--nocapture'],{env,stdio:'inherit'});
}finally{cleanup();}
