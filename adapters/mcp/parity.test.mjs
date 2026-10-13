import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile, mkdtemp, rm } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { resolve, dirname } from 'node:path';
import { tmpdir } from 'node:os';
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';
import { EngineClient } from './engine-client.mjs';
const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '../..');
const binary = process.env.SPRITELAB_ENGINE ?? resolve(root, 'target/debug/spritelab');
async function connect(workspace) {
  const transport = new StdioClientTransport({ command: process.execPath, args: [resolve(here, 'server.mjs'), '--engine', binary, '--workspace', workspace] });
  const client = new Client({ name: 'spritelab-parity-test', version: '0.1.0' });
  await client.connect(transport); return client;
}
function call(client, request) {
  const { command, ...arguments_ } = request;
  return client.callTool({ name: 'spritelab_' + command.replaceAll('.', '_'), arguments: arguments_ });
}
test('actual CLI, Python CLI replay and MCP produce identical state/results/errors', async () => {
  const workspace = await mkdtemp(resolve(tmpdir(), 'spritelab-parity-'));
  const fixture = resolve(root, 'tests/parity.jsonl');
  const source = await readFile(fixture, 'utf8');
  const expected = spawnSync(binary, ['--workspace', workspace], { input: source, encoding: 'utf8' });
  assert.equal(expected.status, 0, expected.stderr);
  const artifactNames = ['parity.spritelab', 'parity.png', 'parity-atlas.png', 'parity.gif'];
  const artifactBytes = await Promise.all(artifactNames.map(name => readFile(resolve(workspace, name))));
  const python = spawnSync('python3', [resolve(root, 'examples/replay.py'), '--engine', binary, '--workspace', workspace, '--transcript', fixture], { encoding: 'utf8' });
  assert.equal(python.status, 0, python.stderr); assert.equal(python.stdout, expected.stdout);
  for (const [i, name] of artifactNames.entries()) assert.deepEqual(await readFile(resolve(workspace, name)), artifactBytes[i]);
  const responses = expected.stdout.trim().split('\n').map(JSON.parse);
  const requests = source.trim().split('\n').map(JSON.parse);
  const client = await connect(workspace);
  try {
    for (let i = 0; i < requests.length; i++) {
      const response = await call(client, requests[i]);
      assert.deepEqual(response.structuredContent, responses[i]);
      assert.equal(response.isError, !responses[i].ok);
    }
    for (const [i, name] of artifactNames.entries()) assert.deepEqual(await readFile(resolve(workspace, name)), artifactBytes[i]);
    assert.equal(responses[13].error.code, 'invalid_arguments');
    assert.deepEqual(responses[11].result.document, responses[14].result.document);
    assert.equal(responses[12].result.rgba.length, 256);
  } finally { await client.close(); await rm(workspace, { recursive: true, force: true }); }
});
test('MCP catalog comes from Rust; preview image and resources expose that session', async () => {
  const workspace = await mkdtemp(resolve(tmpdir(), 'spritelab-mcp-'));
  const client = await connect(workspace);
  try {
    const catalog = JSON.parse(spawnSync(binary, ['--workspace', workspace, '--catalog'], { encoding: 'utf8' }).stdout);
    const tools = (await client.listTools()).tools;
    assert.equal(tools.length, catalog.length);
    for (const command of catalog) {
      const tool = tools.find(t => t.name === 'spritelab_' + command.id.replaceAll('.', '_'));
      assert.deepEqual(tool.inputSchema.properties.args, command.arguments_schema);
    }
    await call(client, { command: 'document.create', document_id: 'preview', args: { width: 4, height: 4 } });
    const preview = await call(client, { command: 'document.preview', document_id: 'preview', args: { frame_id: 2 } });
    assert.equal(preview.content[0].type, 'image'); assert.equal(preview.content[0].mimeType, 'image/png');
    assert.equal(Buffer.from(preview.content[0].data, 'base64').subarray(0, 8).toString('hex'), '89504e470d0a1a0a');
    const resources = await client.listResources(); assert.equal(resources.resources.length, 1);
    const data = await client.readResource({ uri: resources.resources[0].uri });
    assert.equal(JSON.parse(data.contents[0].text).result.document.width, 4);
  } finally { await client.close(); await rm(workspace, { recursive: true, force: true }); }
});
test('concurrent requests are serialized; only one edit with the same revision commits', async () => {
  const workspace = await mkdtemp(resolve(tmpdir(), 'spritelab-concurrency-'));
  const client = await connect(workspace);
  try {
    await call(client, { command: 'document.create', document_id: 'shared', args: { width: 4, height: 4 } });
    const r = await Promise.all(['One', 'Two'].map(name => call(client, {
      command: 'layer.create', document_id: 'shared', expected_revision: 0, args: { name },
    })));
    assert.equal(r.filter(v => v.structuredContent.ok).length, 1);
    assert.equal(r.find(v => !v.structuredContent.ok).structuredContent.error.code, 'revision_conflict');
  } finally { await client.close(); await rm(workspace, { recursive: true, force: true }); }
});
test('adapter child failure rejects pending requests and closes the session', async () => {
  const workspace = await mkdtemp(resolve(tmpdir(), 'spritelab-failure-'));
  const engine = new EngineClient(resolve(workspace, 'absent-engine'), workspace);
  try { await assert.rejects(engine.execute({ command: 'command.list', args: {} }));
    await assert.rejects(engine.execute({ command: 'command.list', args: {} }));
  } finally { engine.close(); await rm(workspace, { recursive: true, force: true }); }
});
