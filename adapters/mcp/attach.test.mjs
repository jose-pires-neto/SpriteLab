import test from 'node:test';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { once } from 'node:events';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve, join } from 'node:path';
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';
import { EngineClient } from './engine-client.mjs';
const binary=resolve('../../target/debug/spritelab');
test('attached MCP and independent CLI client share a real local session and view',async()=>{
 const root=await mkdtemp(join(tmpdir(),'sl-live-'));
 const host=spawn(binary,['--workspace',root,'--host'],{stdio:['ignore','pipe','inherit']});
 const lines=createInterface({input:host.stdout});
 let session,cli,client;let stage="host";
 try{
  session=JSON.parse((await once(lines,'line'))[0]);
  stage="CLI creation";cli=new EngineClient(binary,null,session.socket);
  assert.equal((await cli.execute({command:'document.create',document_id:'live',args:{width:8,height:8}})).ok,true);
  assert.equal((await cli.execute({command:'editor.configure',args:{document_id:'live',frame_id:2,layer_id:1}})).ok,true);
  stage="MCP initialize";client=new Client({name:'attach-test',version:'1.0.0'});
  await client.connect(new StdioClientTransport({command:process.execPath,args:[resolve('server.mjs'),'--engine',binary,'--workspace',root,'--attach-latest'],stderr:'inherit'}));
  const call=(command,args,revision)=>client.callTool({name:'spritelab_'+command.replaceAll('.','_'),arguments:{document_id:'live',args,...(revision!==undefined?{expected_revision:revision}:{})}});
  stage="MCP rectangle";const response=await call('canvas.rectangle',{layer_id:1,frame_id:2,rect:{x:1,y:1,width:3,height:3},filled:true,color:[255,0,0,255]},0);
  assert.equal(response.structuredContent.ok,true);
  stage="CLI snapshot";const inspected=await cli.execute({command:'document.snapshot',document_id:'live',args:{frame_id:2}});
  assert.equal(inspected.revision,1);assert.equal(inspected.result.rgba[(1*8+1)*4],255);
  stage="MCP stale edit";const stale=await call('canvas.stroke',{layer_id:1,frame_id:2,points:[[0,0]],color:[0,0,0,255]},0);assert.equal(stale.structuredContent.error.code,'revision_conflict');
  stage="MCP view";const view=await client.callTool({name:'spritelab_editor_configure',arguments:{args:{tool:'ellipse',zoom:8,onion_skin:true,color:[12,34,56,255],filled:true}}});assert.equal(view.structuredContent.ok,true);
  const shared=await cli.execute({command:'editor.inspect',args:{}});assert.equal(shared.result.view.tool,'ellipse');assert.equal(shared.result.view.zoom,8);assert.equal(shared.result.document_revision,1);assert.deepEqual(shared.result.view.color,[12,34,56,255]);assert.equal(shared.result.view.filled,true);assert.equal(shared.result.dirty,true);
  stage='MCP save';assert.equal((await call('document.save',{path:'live.spritelab'})).structuredContent.ok,true);
  const saved=await cli.execute({command:'editor.inspect',args:{}});assert.equal(saved.result.dirty,false);assert.equal(saved.result.document_path,'live.spritelab');
  await client.close();client=null;
  assert.equal((await cli.execute({command:'document.list',args:{}})).result.documents.length,1);
 }catch(e){throw new Error(stage+": "+e.message,{cause:e})}finally{await client?.close();cli?.close();host.kill();await once(host,'exit');lines.close();if(session){await rm(session.socket,{force:true});}await rm(root,{recursive:true,force:true});}
});
