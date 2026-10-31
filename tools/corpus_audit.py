#!/usr/bin/env python3
"""One-off/manual audit. Downloads fixtures at a pinned revision, never edits originals."""
import os,base64,collections,hashlib,json,re,subprocess,sys,urllib.request,zipfile
from pathlib import Path
import xml.etree.ElementTree as ET
from PIL import Image,ImageChops
ROOT=Path("audit-results");WORK=Path("audit-work");ROOT.mkdir(exist_ok=True);WORK.mkdir(exist_ok=True)
manifest=json.loads(Path("tools/corpus-manifest.json").read_text())
W="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
T="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
def norm(s): return " ".join(s.replace("\ufffc","").split())
def tokens(s): return collections.Counter(re.findall(r"\w+",norm(s),re.UNICODE))
def source_text(path):
 with zipfile.ZipFile(path) as z:
  texts=[]
  if path.suffix==".docx":
   for n in z.namelist():
    if re.fullmatch(r"word/(document|header\d+|footer\d+|footnotes|endnotes)\.xml",n):
     root=ET.fromstring(z.read(n))
     def paragraph_text(e):
      if e.tag=="{"+W+"}t":return e.text or ""
      return "".join(paragraph_text(c) for c in e if c.tag!="{"+W+"}p")
     texts.extend(paragraph_text(e) for e in root.iter("{"+W+"}p"))
  else:
   for n in ["content.xml","styles.xml"]:
    root=ET.fromstring(z.read(n))
    for e in root.iter():
     if e.tag in ["{"+T+"}p","{"+T+"}h"]:texts.append("".join(e.itertext()))
  return tokens(" ".join(texts))
def walk(bs):
 for b in bs:
  yield b
  if b["kind"]=="table":
   for r in b["rows"]:
    for c in r["cells"]:yield from walk(c["blocks"])
def signature(s):
 stories=[("body",s["body"])]
 stories+=sorted([(p["kind"],p["blocks"]) for p in s["parts"]],key=lambda x:(x[0],json.dumps(x[1],sort_keys=True)))
 ps=[b for _,bs in stories for b in walk(bs) if b["kind"]=="para"]
 tables=[b for _,bs in stories for b in walk(bs) if b["kind"]=="table"]
 chars=[]
 for p in ps:
  runs=[]
  for r in p["runs"]:
   if not r["text"]:continue
   if runs and runs[-1]["format"]==r["format"]:runs[-1]["text"]+=r["text"]
   else:runs.append(dict(r))
  chars.append(runs)
 media=s["model"].get("media",{})
 hashes=sorted(hashlib.sha256(base64.b64decode(v)).hexdigest() for v in media.values())
 sections=[s["model"]["lastSection"]]
 for _,bs in stories:
  pass
 # Read physical section properties from raw model, including section breaks.
 def rawwalk(bs):
  for b in bs:
   if b.get("kind")=="para" and b.get("section"):yield b["section"]
   if b.get("kind")=="table":
    for r in b["rows"]:
     for c in r["cells"]:yield from rawwalk(c["blocks"])
 sections=list(rawwalk(s["model"]["body"]))+sections
 def physical(p):return {k:v for k,v in p.items() if not any(x in k.lower() for x in ["header","footer"])}
 return {"text":[(kind,[norm(b["text"]) for b in walk(bs) if b["kind"]=="para"]) for kind,bs in stories],
 "character_format":chars,"paragraph_format":[p["format"] for p in ps],
 "tables":[{"grid":t["grid"],"props":t["props"],"rows":[{"props":r["props"],"cells":[c["props"] for c in r["cells"]]} for r in t["rows"]]} for t in tables],
 "objects":[p["objects"] for p in ps],"media_bytes":hashes,"page_properties":[physical(x) for x in sections],
 "page_count":s["pages"]}
def diff(a,b,path="",out=None):
 if out is None:out=[]
 if len(out)>=60:return out
 if isinstance(a,(int,float)) and not isinstance(a,bool) and isinstance(b,(int,float)):
  if abs(a-b)<=0.001:return out
 elif type(a)==type(b) and isinstance(a,dict):
  for k in sorted(set(a)|set(b)):diff(a.get(k),b.get(k),path+"/"+str(k),out)
  return out
 elif type(a)==type(b) and isinstance(a,list):
  if len(a)!=len(b):out.append({"path":path+"/length","before":len(a),"after":len(b)})
  for i,(x,y) in enumerate(zip(a,b)):diff(x,y,path+"/"+str(i),out)
  return out
 elif a==b:return out
 out.append({"path":path,"before":str(a)[:180],"after":str(b)[:180]});return out
results=[]
for ix,fixture in enumerate(manifest["files"]):
 name=Path(fixture["path"]).name;stem=f"{ix+1:02}-{Path(name).stem}";folder=ROOT/stem;folder.mkdir()
 item={"file":name,"source_path":fixture["path"],"source_blob_sha":fixture["blob_sha"],"status":"error"}
 try:
  url=f'https://raw.githubusercontent.com/{manifest["repository"]}/{manifest["revision"]}/{fixture["path"]}'
  data=urllib.request.urlopen(url,timeout=40).read()
  assert hashlib.sha1(b"blob "+str(len(data)).encode()+b"\0"+data).hexdigest()==fixture["blob_sha"]
  source=WORK/name;source.write_bytes(data);item["source_sha256"]=hashlib.sha256(data).hexdigest()
  proc=subprocess.run(["target/release/examples/corpus_audit",str(source),str(folder)],capture_output=True,text=True,timeout=90)
  (folder/"worker.log").write_text(proc.stdout+proc.stderr)
  if proc.returncode:raise RuntimeError(f"worker exit {proc.returncode}: "+proc.stderr[-400:])
  before=json.loads((folder/"before.json").read_text());after=json.loads((folder/"after.json").read_text())
  a,b=signature(before),signature(after)
  changes={k:diff(a[k],b[k]) for k in a if diff(a[k],b[k])}
  item["changes"]=changes;item["pages"]=[a["page_count"],b["page_count"]]
  item["table_counts"]=[len(a["tables"]),len(b["tables"])]
  item["part_counts"]=[len(before["parts"]),len(after["parts"])]
  item["body_text_unchanged"]=norm(before["body_text"])==norm(after["body_text"])
  item["media_counts"]=[len(a["media_bytes"]),len(b["media_bytes"])]
  original=source_text(source);imported=tokens(before["body_text"]+" "+" ".join(p["text"] for p in before["parts"]))
  reopened=tokens(after["body_text"]+" "+" ".join(p["text"] for p in after["parts"]))
  item["source_tokens"]=sum(original.values());item["import_missing_tokens"]=dict(original-imported)
  item["roundtrip_missing_tokens"]=dict(imported-reopened)
  item["roundtrip_added_tokens"]=dict(reopened-imported)
  item["render"]=[]
  for page in range(1,min(before["pages"],after["pages"],2)+1):
   x=Image.open(folder/f"before-{page}.png").convert("RGBA");y=Image.open(folder/f"after-{page}.png").convert("RGBA")
   same=x.size==y.size and x.tobytes()==y.tobytes()
   entry={"page":page,"before_size":x.size,"after_size":y.size,"identical":same}
   if x.size==y.size:
    delta=ImageChops.difference(x.convert("RGB"),y.convert("RGB"))
    entry["changed_pixels"]=sum(1 for p in delta.getdata() if max(p)>2)
    delta.save(folder/f"diff-{page}.png")
   item["render"].append(entry)
  item["status"]="unchanged" if not changes and all(p["identical"] for p in item["render"]) else "differences"
  # Keep evidence and provenance, but do not redistribute imported/exported fixture documents or their full model.
  for n in ["before.json","after.json"]+([] if os.environ.get("AUDIT_KEEP_SAVED") else [f"saved{source.suffix}"]):(folder/n).unlink(missing_ok=True)
 except Exception as e:item["error"]=str(e)
 results.append(item);print(name,item["status"],flush=True)
(ROOT/"results.json").write_text(json.dumps({"manifest":manifest,"results":results},ensure_ascii=False,indent=2))
lines=["# One-off DOCX/ODT corpus audit","",f'LibreOffice revision: `{manifest["revision"]}`. {len(results)} fixtures. Original files were not modified. No release gate was added.',"",
"Import token differences are diagnostic flags, not proof of loss: generated fields, repeated/header stories and XML text boundaries can differ. Round-trip semantic differences and first-two-page raster comparisons are separate checks. Float tolerance is 0.001 points; differing style/list/part IDs and equivalent serializations require inspection. No Word/LibreOffice visual baseline is claimed.","",
"| File | Open/save/reopen | Changed semantic categories | Pages | First 2 pages identical | Source token flags |",
"|---|---|---|---|---|---|"]
for r in results:
 lines.append("| "+r["file"]+" | "+r["status"]+" | "+", ".join(r.get("changes",{}))+" | "+str(r.get("pages","—"))+" | "+str(all(x["identical"] for x in r.get("render",[])))+" | "+str(sum(r.get("import_missing_tokens",{}).values()))+" |")
lines+=["","Detailed, capped property differences and token flags are in results.json. Per-case images and worker logs accompany the Actions artifact. Count a case as fully measured-unchanged only if all compared semantic categories and rendered pages match; that still does not establish source import fidelity or full-document visual fidelity.","","## Provenance and licensing","","Fixtures are downloaded for this audit from LibreOffice/core at the pinned revision and verified against their Git blob SHA. LibreOffice has MPL/LGPL and other applicable notices; fixture-specific provenance may vary. This repository does not relicense or vendor the fixtures. Source links and project notices: https://www.libreoffice.org/licenses/ and https://github.com/LibreOffice/core/tree/"+manifest["revision"]+". Output images are derived test evidence; retain original attribution/notices when redistributing. The artifact contains the upstream COPYING, COPYING.LGPL and COPYING.MPL notices.",""]
(ROOT/"REPORT.md").write_text("\n".join(lines))
for name in ["COPYING","COPYING.LGPL","COPYING.MPL"]:
 (ROOT/name).write_bytes(urllib.request.urlopen(f'https://raw.githubusercontent.com/{manifest["repository"]}/{manifest["revision"]}/{name}',timeout=40).read())
