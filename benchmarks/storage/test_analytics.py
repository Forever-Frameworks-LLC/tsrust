"""Small synthetic fixtures only; no benchmark execution."""
import hashlib
import json
import sqlite3
import tempfile
import unittest
from pathlib import Path
import analytics
import store

class AnalyticsTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(); self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name); self.db = self.root/'records.sqlite'
        self.row = {'id':'array_pop_int','sourceSha256':'a'*64,'counterpartSha256':'b'*64,'iterations':10,'operationsPerIteration':2,'result':'string[]','notes':'Full setup/removal/retention pipeline'}
        self.raw = {'schemaVersion':1,'timestamp':'2026-01-01T00:00:00Z','semanticEpoch':'fixture','measurementScope':'execution-only','verifiedOnly':False,'machine':{'cpu':'fixture'},'versions':{'node':'fixture'},'manifest':{'workloads':[self.row]},'sources':[{'id':'array_pop_int','tsrSha256':'a'*64,'tsSha256':'b'*64,'emittedSha256':'c'*64}],
                    'correctness':{'exactBits':'hé😀\u0000'},'execution':[{'workload':'array_pop_int','engine':engine,'processIndex':process,'durations':durations,'stdout':'original unmodified evidence'} for engine,groups in [('native',[[2,4],[1,8]]),('node-tsc',[[4,8],[2,16]])] for process,durations in enumerate(groups)]}
        sample = {'allocations':4,'reallocations':1,'requestedBytes':100,'retainedRequestedBytes':80,'peakRequestedBytes':120,'inputRequestedBytes':12,'liveBytesAfterOutputDrop':0,'liveBytesAfterDrop':0}
        self.profile = {'timestamp':self.raw['timestamp'],'semanticEpoch':'fixture','kind':'native-workload-allocation-diagnosis','manifest':self.raw['manifest'],'rows':[{'workload':'array_pop_int','sourceSha256':'a'*64,'iterations':10,'samples':[sample]*3}]}
        self.paths = ['runtime/results/run.json','runtime/diagnostics/profile.json']
        for name,data in zip(self.paths,[self.raw,self.profile]):
            p=self.root/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
        self.proof = self.root/'runtime/diagnostics/proof.json'
        self.proof.write_text(json.dumps({'runtimeAccepted':True,'profilesAccepted':True,'timed':True,'records':[{'path':str(self.root/name),'sha256':analytics.sha(self.root/name)} for name in self.paths]}))

    def ingest(self):
        return analytics.ingest(self.db,self.root,self.proof,analytics.sha(self.proof))

    def test_exact_bytes_reciprocal_rates_source_keys_and_profile_operations(self):
        result=self.ingest();self.assertEqual([x['samples'] for x in result['indexed']],[8,3])
        out=self.root/'restored';out.mkdir()
        for name in self.paths:
            store.export_record(self.db,name,out);self.assertEqual((out/name).read_bytes(),(self.root/name).read_bytes())
        query=analytics.checkpoint(self.db);self.assertEqual(query['nativeWins'],1);row=query['rows'][0]
        self.assertEqual(row['sourceSha256'],'a'*64);self.assertEqual(row['operationsPerBatch'],20)
        self.assertEqual(row['native'],{'samples':4,'min':2500.0,'avg':9375.0,'max':20000.0});self.assertEqual(row['nativeNodeRatio'],2)
        self.assertEqual(query['semanticEpoch'],'fixture');self.assertEqual(query['measurementScope'],'execution-only')
        with store.connect(self.db) as db:
            self.assertEqual(db.execute('SELECT process_index,sample_index FROM analytics_samples WHERE engine="native" ORDER BY process_index,sample_index').fetchall(),[(0,0),(0,1),(1,0),(1,1)])
            self.assertEqual(db.execute('SELECT operations,live_after_output,live_after_final FROM analytics_profiles').fetchall(),[(20,0,0)]*3)
            self.assertEqual(db.execute('SELECT allocations_per_operation,retained_batch_peak_bytes_per_operation,input_adaptation_bytes FROM analytics_profile_resources').fetchall(),[(0.2,6.0,12)]*3)

    def test_idempotence_and_immutable_original_conflict(self):
        self.ingest();result=self.ingest();self.assertTrue(all(x['alreadyIndexed'] for x in result['indexed']))
        with store.connect(self.db,writable=True) as db:
            with self.assertRaises(sqlite3.IntegrityError):db.execute('UPDATE records SET kind="changed"')
        (self.root/self.paths[0]).write_text('changed')
        with self.assertRaisesRegex(ValueError,'drift'):self.ingest()
        with store.connect(self.db) as db:self.assertEqual(db.execute('SELECT count(*) FROM analytics_samples').fetchone()[0],8)

    def test_correctness_only_never_becomes_measured(self):
        self.raw['verifiedOnly']=True;self.raw['measurementScope']='correctness-only'
        p=self.root/self.paths[0];p.write_text(json.dumps(self.raw));proof=json.loads(self.proof.read_bytes());proof['records'][0]['sha256']=analytics.sha(p);self.proof.write_text(json.dumps(proof))
        with self.assertRaisesRegex(ValueError,'measured'):self.ingest()
        with self.assertRaisesRegex((ValueError,sqlite3.OperationalError),'no accepted|no such'):analytics.checkpoint(self.db)

if __name__=='__main__':unittest.main()
