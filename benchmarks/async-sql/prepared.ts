declare function require(name:string):any;
declare const process:any;
const driver=process.env.TSRUST_SQL_DRIVER;
const url=process.env.TSRUST_SQL_URL;
async function main():Promise<void>{
 let db:any;
 if(driver==='postgres'){db=new (require('pg').Client)({connectionString:url});await db.connect();}
 else if(driver==='mysql'){db=await require('mysql2/promise').createConnection(url);}
 else db=new (require('node:sqlite').DatabaseSync)(':memory:');
 const statements=new Map<string,any>();
 const query=async(sql:string,parameters:any[]=[]):Promise<any[]>=>{
  if(driver==='postgres')return(await db.query({name:require('node:crypto').createHash('sha1').update(sql).digest('hex'),text:sql,values:parameters})).rows;
  if(driver==='mysql')return(await (/^(BEGIN|COMMIT|ROLLBACK)$/.test(sql)?db.query(sql):db.execute(sql,parameters)))[0];
  let statement=statements.get(sql);if(!statement){statement=db.prepare(sql);statements.set(sql,statement);}return /^SELECT/.test(sql)?statement.all(...parameters):(statement.run(...parameters),[]);
 };
 const insert=driver==='postgres'?'INSERT INTO tsrust_s18_bench VALUES ($1, $2)':'INSERT INTO tsrust_s18_bench VALUES (?, ?)';
 const aggregate=`SELECT CAST(SUM(value) AS ${driver==='mysql'?'SIGNED':'BIGINT'}) AS total FROM tsrust_s18_bench`;
 const batch=async()=>{
  await query('BEGIN');
  try{await query('DELETE FROM tsrust_s18_bench');for(let index=0;index<128;index++)await query(insert,[index,index*2]);await query('COMMIT');}
  catch(error){await query('ROLLBACK');throw error;}
  return Number((await query(aggregate))[0].total);
 };
 try{
  await query('CREATE TABLE tsrust_s18_bench(id BIGINT PRIMARY KEY, value BIGINT NOT NULL)');
  for(let warmup=0;warmup<8;warmup++)await batch();
  for(let sample=0;sample<31;sample++){const start=performance.now();const checksum=await batch();console.log(checksum);console.log(performance.now()-start);}
 }finally{await query('DROP TABLE IF EXISTS tsrust_s18_bench');if(driver==='sqlite')db.close();else await db.end();}
}
main().catch((error:any)=>{console.error(error);process.exitCode=1;});
