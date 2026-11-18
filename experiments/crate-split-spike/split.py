import re,sys,json
sys.path.insert(0,'/Users/seancotter/Desktop/dev/learning/tsrust/tools')
import importlib.util
spec=importlib.util.spec_from_file_location('csa','/Users/seancotter/Desktop/dev/learning/tsrust/tools/crate-split-analysis.py'); csa=importlib.util.module_from_spec(spec); spec.loader.exec_module(csa)
src=open(sys.argv[1]).read(); members=set(json.loads(sys.argv[2]))
# 1. undo chunking
lines=src.split('\n'); out=[]
for l in lines:
    if re.match(r'^mod tsrust_generated_unit\d+ \{ use super::\*;$',l): continue
    if re.match(r'^use tsrust_generated_unit\d+::\*;$',l): continue
    if l=='}' and out and False: pass
    out.append(l)
src='\n'.join(out)
# chunk closers: a lone "}" line that directly preceded "use tsrust_generated_unitN" -- remove by re-scan of original
orig=lines; out=[]; skip_close=set()
for i,l in enumerate(orig):
    if re.match(r'^use tsrust_generated_unit\d+::\*;$',l) and i>0 and orig[i-1]=='}': skip_close.add(i-1)
out=[l for i,l in enumerate(orig) if i not in skip_close and not re.match(r'^mod tsrust_generated_unit\d+ \{ use super::\*;$',l) and not re.match(r'^use tsrust_generated_unit\d+::\*;$',l)]
src='\n'.join(l[len('pub(super) '):] if l.startswith('pub(super) ') else l for l in out)
# 2. split items
items=csa.items(src)
rest=[];edit=[]
for it in items:
    kind,name=csa.owner(it)
    (edit if name in members else rest).append(it)

def at_depth(text, depth_wanted, keywords):
    """Insert 'pub ' before item keywords that start at the given brace depth (strings/chars skipped)."""
    out=[];i=0;n=len(text);depth=0;last=''
    starts=('{',';','}','\n',' ','\t')
    while i<n:
        c=text[i]
        if c=='"':
            j=i+1
            while j<n and text[j]!='"':
                j+=2 if text[j]=='\\' else 1
            out.append(text[i:j+1]);i=j+1;continue
        if c=="'" and i+2<n and (text[i+2]=="'" or (text[i+1]=='\\')):
            j=text.find("'",i+2) if text[i+1]=='\\' else i+2
            out.append(text[i:j+1]);i=j+1;continue
        if c=='{': depth+=1
        elif c=='}': depth-=1
        if depth==depth_wanted and (i==0 or text[i-1] in starts):
            for kw in keywords:
                if text.startswith(kw,i) and not any(text[max(0,i-12):i].endswith(q) for q in ('pub ','async ','unsafe ','const ',') ')):
                    out.append('pub ');break
        out.append(c);i+=1
    return ''.join(out)

def publicize(it):
    head=it.split('\n',1)[0]
    kind,name=csa.owner(it)
    if kind=='impl':
        if ' for ' not in head.split('{')[0]:
            it=at_depth(it,1,('async fn ','fn ','const '))
        return it
    if kind in ('fn','struct','enum','union','type','trait','mod','const','static','use') and not it.startswith('pub'):
        it='pub '+it
    if kind=='struct':
        first,_,restt=it.partition('\n')
        def fields(m):
            body=m.group(0)
            return re.sub(r'([{,]\s*)(?!pub\b)([a-z_][a-z0-9_]*)\s*:(?!:)',r'\1pub \2:',body)
        it=re.sub(r'\{[^{}]*\}',fields,it,count=1)
        it=re.sub(r'^(pub struct [A-Za-z0-9_]+(?:<[^>]*>)?)\((?!pub)',r'\1(pub ',it)
    if kind=='mod':
        it=at_depth(it,1,('fn ','async fn ','struct ','enum ','const ','static ','type ','trait ','mod ','use '))
    return it
rest=[publicize(i) for i in rest]
edit=[publicize(i) for i in edit]
mainidx=[i for i,it in enumerate(rest) if re.match(r'pub fn main\(',it)]
for i in mainidx: rest[i]=rest[i].replace('pub fn main(','pub fn tsrust_real_main(',1)
rest_text='\n'.join(rest)
rest_text=re.sub(r'(?m)^((?:#\[[^\]\n]*\]\s*)+)(?=(?:enum|struct|fn|type|trait|const|static|union|mod) )',r'\1pub ',rest_text)
open(sys.argv[3]+'/rest.rs','w').write(rest_text)
edit_src='#![allow(unused)]\nuse rest::*;\n'+'\n'.join(e.replace('crate::','rest::') for e in edit)+'\nfn main()->std::process::ExitCode{ rest::tsrust_real_main() }\n'
open(sys.argv[3]+'/edit.rs','w').write(edit_src)
print('rest items',len(rest),'edit items',len(edit),'edit bytes',len(edit_src))
