import unittest
from json_metrics import summarize

class Metrics(unittest.TestCase):
 def test_rates_and_nearest_rank(self):
  result=summarize([{'durations':list(range(1,32)), 'iterations':100}]*3,20)
  self.assertEqual(result['p50BatchMs'],16)
  self.assertEqual(result['p95BatchMs'],30)
  self.assertEqual(result['medianOfferedBytesPerSecond'],125000)
  self.assertEqual(summarize([{'durations':[1]*31,'iterations':1}]*3,0)['medianOfferedBytesPerSecond'],0)
 def test_incomplete_or_invalid_rejected(self):
  for samples in [[],[{'durations':[1], 'iterations':1}]*3,[{'durations':[0]*31,'iterations':1}]*3]:
   with self.assertRaises(ValueError):summarize(samples,1)
if __name__=='__main__':unittest.main()
