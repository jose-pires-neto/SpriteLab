// Demonstrates actual MCP editing in a running desktop session.
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';
import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
const discovery=JSON.parse(await readFile(process.argv[2],'utf8'));
const client=new Client({name:'SpriteLab live demo',version:'0.3.0'});
await client.connect(new StdioClientTransport({command:process.execPath,args:[resolve(import.meta.dirname,'server.mjs'),'--engine',resolve(import.meta.dirname,'../../target/debug/spritelab'),'--attach',discovery.socket],stderr:'inherit'}));
const records=[];
const call=async(command,args={},doc='mcp-validation')=>{const result=await client.callTool({name:'spritelab_'+command.replaceAll('.','_'),arguments:{...(doc?{document_id:doc}:{}),args}});records.push({command,response:result.structuredContent});if(!result.structuredContent.ok)throw Error(result.structuredContent.error.message);return result.structuredContent;};
try{
 await call('document.open',{path:'exemplo-cogumelo.spritelab'});
 const inspected=await call('document.inspect');const d=inspected.result.document;const frame=d.frames[0].id;
 const layer=await call('layer.create',{name:'Desenhado pelo MCP'});const lid=layer.result.results[0].layer_id;
 await call('canvas.rectangle',{layer_id:lid,frame_id:frame,rect:{x:1,y:1,width:7,height:5},color:[85,180,200,255],filled:false});
 await call('canvas.stroke',{layer_id:lid,frame_id:frame,points:[[2,3],[6,3]],color:[255,220,100,255],size:1});
 await call('editor.configure',{document_id:'mcp-validation',frame_id:frame,layer_id:lid,tool:'rectangle',grid:true,onion_skin:true,zoom:12},null);
 await call('document.save',{path:'validacao-mcp-desktop.spritelab',overwrite:true});
 await call('export.png',{path:'validacao-mcp-desktop.png',frame_id:frame,overwrite:true});
 await call('export.gif',{path:'validacao-mcp-desktop.gif',overwrite:true});
 await call('document.snapshot',{frame_id:frame});
 await writeFile(resolve(import.meta.dirname,'../../docs/live-desktop-mcp.json'),JSON.stringify({transport:'Official MCP SDK stdio -> CLI --attach -> desktop-owned Unix session',discovery,records},null,2)+'\n');
 console.log('MCP edited the desktop session, exported PNG/GIF and saved an editable project.');
}finally{await client.close()}
