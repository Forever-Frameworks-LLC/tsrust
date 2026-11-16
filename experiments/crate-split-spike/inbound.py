import sys,re,collections,json,importlib.util
spec=importlib.util.spec_from_file_location('csa','/Users/seancotter/Desktop/dev/learning/tsrust/tools/crate-split-analysis.py'); csa=importlib.util.module_from_spec(spec); spec.loader.exec_module(csa)
text=open(sys.argv[1]).read()
groups=collections.defaultdict(list)
for it in csa.items(text):
    k,n=csa.owner(it)
    if n: groups[n].append(it)
names=set(groups); files={}; refs={}
for n,b in groups.items():
    j='\n'.join(b); f=csa.POSITION.findall(j)
    if f: files[n]=collections.Counter(f).most_common(1)[0][0]
    refs[n]={t for t in csa.IDENT.findall(j) if t in names and t!=n}
def is_fn(n): return re.fullmatch(r'(tsr_result_function|function)\d+',n) is not None
def is_async(n): return any(re.search(r'^(pub )?async fn '+n+r'\b',b) for b in groups[n])
byfile=collections.defaultdict(set)
for n,f in files.items(): byfile[f].add(n)
rows=[]
async_backedges=0; total_cross=0
for f,members in byfile.items():
    inbound=collections.Counter(); kinds=collections.Counter()
    for n,r in refs.items():
        if n in members: continue
        for t in r & members:
            inbound[t]+=1
    nonfn=[t for t in inbound if not is_fn(t)]
    asyncs=[t for t in inbound if is_fn(t) and is_async(t)]
    size=sum(len('\n'.join(groups[m])) for m in members)
    rows.append((len(nonfn),len(asyncs),size,f.split('/apps/studio/')[-1],len(inbound),sorted(members)))
rows.sort(key=lambda r:(r[0],r[1],-r[2]))
for r in rows[:12]: print(r[0],r[1],r[2],r[3],'inbound',r[4],'members',len(r[5]))
json.dump([r for r in rows],open(sys.argv[2],'w'))
