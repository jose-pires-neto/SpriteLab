import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  ListToolsRequestSchema, CallToolRequestSchema, ListResourcesRequestSchema,
  ReadResourceRequestSchema, ListResourceTemplatesRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';
import { EngineClient } from './engine-client.mjs';
import { resolve } from 'node:path';
import {readFileSync,readdirSync,statSync,existsSync} from 'node:fs';

const args = process.argv.slice(2);
const option = key => args[args.indexOf(key) + 1];
if (!args.includes('--engine') || (!args.includes('--workspace') && !args.includes('--attach'))) {
  console.error('Usage: node server.mjs --engine /absolute/path/spritelab --workspace /absolute/project [--attach-latest] | --attach SOCKET');
  process.exit(1);
}
let socket=args.includes('--attach')?resolve(option('--attach')):null;
if(args.includes('--attach-latest')){
 const root=resolve(option('--workspace'));
 const candidates=readdirSync(root).filter(n=>/^\.spritelab-session-\d+\.json$/.test(n)).map(n=>({path:resolve(root,n),time:statSync(resolve(root,n)).mtimeMs})).sort((a,b)=>b.time-a.time);
 for(const entry of candidates){try{const info=JSON.parse(readFileSync(entry.path,'utf8'));if(info.protocol_version!==1||!existsSync(info.socket))continue;process.kill(info.pid,0);if(statSync(info.socket).isSocket()){socket=info.socket;break}}catch{}}
 if(!socket){console.error('No live SpriteLab session in workspace. Open the desktop app first, or omit --attach-latest for explicit headless mode.');process.exit(1);}
}
const engine = new EngineClient(resolve(option('--engine')), args.includes('--workspace') ? resolve(option('--workspace')) : null, socket);
const server = new Server({ name: 'spritelab', version: '0.3.1' }, {
  capabilities: { tools: {}, resources: {} },
  instructions: (socket ? 'Attached to the live desktop Rust session. ' : 'Isolated headless session. ') + 'All operations execute in the selected Rust engine session. Inspect IDs and revision; use expected_revision for edits. Batches contain only edits to one document. Paths are workspace-relative. Live attachment is explicit via --attach SOCKET or --attach-latest --workspace ROOT.',
});
try {
  const response = await engine.execute({ command: 'command.list', args: {} });
  if (!response.ok) throw new Error(response.error.message);
  const catalog = response.result.commands;
  const byTool = new Map(catalog.map(command => ['spritelab_' + command.id.replaceAll('.', '_'), command]));
  server.setRequestHandler(ListToolsRequestSchema, async () => ({ tools: [...byTool].map(([name, d]) => {
    const global = ['command.list', 'document.list','editor.inspect','editor.configure','workspace.inspect'].includes(d.id);
    const definitions = d.arguments_schema.definitions;
    const inputSchema = {
      type: 'object', additionalProperties: false,
      properties: {
        request_id: { type: 'string', maxLength: 128 },
        document_id: { type: 'string', pattern: '^[A-Za-z0-9_-]{1,64}$' },
        expected_revision: { type: 'integer', minimum: 0 },
        args: d.arguments_schema,
      }, required: global ? ['args'] : ['document_id', 'args'],
      ...(definitions && typeof definitions === 'object' ? { definitions } : {}),
    };
    return { name, description: d.summary, inputSchema, annotations: {
      readOnlyHint: d.effect === 'query', destructiveHint: d.id === 'document.close' || d.effect === 'file_write' || d.effect === 'edit' || d.effect === 'history',
      idempotentHint: d.effect === 'query', openWorldHint: false,
    } };
  }) }));
  server.setRequestHandler(CallToolRequestSchema, async request => {
    const d = byTool.get(request.params.name);
    if (!d) return { isError: true, content: [{ type: 'text', text: 'Unknown SpriteLab tool' }] };
    const response = await engine.execute({ ...request.params.arguments, command: d.id });
    const content = [];
    if (response.ok && d.id === 'document.preview') {
      content.push({ type: 'image', data: response.result.png_base64, mimeType: 'image/png' });
      const { png_base64, ...metadata } = response.result; response.result = metadata;
    }
    content.push({ type: 'text', text: JSON.stringify(response) });
    return { isError: !response.ok, content, structuredContent: response };
  });
  server.setRequestHandler(ListResourcesRequestSchema, async () => {
    const r = await engine.execute({ command: 'document.list', args: {} });
    return { resources: r.result.documents.map(d => ({
      uri: `spritelab://documents/${d.document_id}`, name: d.document_id,
      description: `Authoritative document at revision ${d.revision}`, mimeType: 'application/json',
    })) };
  });
  server.setRequestHandler(ListResourceTemplatesRequestSchema, async () => ({ resourceTemplates: [{
    uriTemplate: 'spritelab://documents/{document_id}', name: 'SpriteLab document', mimeType: 'application/json',
  }] }));
  server.setRequestHandler(ReadResourceRequestSchema, async request => {
    const match = /^spritelab:\/\/documents\/([A-Za-z0-9_-]{1,64})$/.exec(request.params.uri);
    if (!match) throw new Error('Unknown resource URI');
    const r = await engine.execute({ command: 'document.inspect', document_id: match[1], args: {} });
    if (!r.ok) throw new Error(r.error.message);
    return { contents: [{ uri: request.params.uri, mimeType: 'application/json', text: JSON.stringify(r) }] };
  });
  server.onclose = () => engine.close();
  for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, () => { engine.close(); server.close().finally(() => process.exit(0)); });
  await server.connect(new StdioServerTransport());
} catch (error) { engine.close(); console.error(error); process.exit(1); }
