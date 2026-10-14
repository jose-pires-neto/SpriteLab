#!/usr/bin/env python3
"""Measure JSONL round-trip edit/render latency; no production speed guarantee."""
import argparse, json, platform, statistics, subprocess, tempfile, time
p=argparse.ArgumentParser();p.add_argument('--engine',required=True);args=p.parse_args()
with tempfile.TemporaryDirectory(prefix='spritelab-bench-') as workspace:
    engine=subprocess.Popen([args.engine,'--workspace',workspace],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
    revision=0
    def call(command, parameters, guarded=False):
        global revision
        request={'command':command,'document_id':'benchmark','args':parameters}
        if guarded:request['expected_revision']=revision
        started=time.perf_counter()
        engine.stdin.write(json.dumps(request)+'\n');engine.stdin.flush()
        result=json.loads(engine.stdout.readline());elapsed=(time.perf_counter()-started)*1000
        if not result['ok']:raise RuntimeError(result)
        revision=result.get('revision',revision)
        return result,elapsed
    try:
        call('document.create',{'width':128,'height':128})
        call('canvas.patch',{'layer_id':1,'frame_id':2,'x':0,'y':0,'width':128,'height':128,'pixels':[[20,100,80,255]]*(128*128)},True)
        second,_=call('layer.create',{'name':'Highlights'},True);layer=second['result']['results'][0]['layer_id']
        call('canvas.patch',{'layer_id':layer,'frame_id':2,'x':0,'y':0,'width':128,'height':128,'pixels':[[100,240,200,80]]*(128*128)},True)
        for _ in range(15):call('frame.add',{'source_frame_id':2,'duration_ms':100},True)
        edits=[];renders=[]
        for i in range(40):
            _,elapsed=call('canvas.stroke',{'layer_id':layer,'frame_id':2,'points':[[i%128,i%128]],'color':[i,200,100,255]},True);edits.append(elapsed)
            _,elapsed=call('document.preview',{'frame_id':2});renders.append(elapsed)
        call('document.save',{'path':'benchmark.spritelab'})
        def summary(values):return {'median_ms':round(statistics.median(values),3),'p95_ms':round(sorted(values)[int(len(values)*.95)-1],3)}
        print(json.dumps({'platform':platform.platform(),'build':'debug','workload':{'width':128,'height':128,'layers':2,'frames':16,'allocated_cel_pixels':524288,'samples':40},'includes':'JSONL serialization and round-trip; no GUI/MCP/network','edit':summary(edits),'preview':summary(renders)},indent=2))
    finally:
        engine.stdin.close();engine.wait(timeout=10)
