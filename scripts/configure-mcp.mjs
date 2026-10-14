#!/usr/bin/env node
// Register a source checkout or a portable release; never infer a shared headless session.
import { existsSync, realpathSync, mkdirSync } from 'node:fs';
import { resolve, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
const root = realpathSync(resolve(dirname(fileURLToPath(import.meta.url)), '..'));
const argv = process.argv.slice(2);
const option = name => { const i = argv.indexOf(name); if (i < 0) return undefined; if (!argv[i + 1] || argv[i + 1].startsWith('--')) throw Error(`${name} requires a value`); return argv[i + 1]; };
for (const arg of argv.filter(v => v.startsWith('--'))) if (!['--workspace', '--print', '--help', '--codex'].includes(arg)) throw Error(`Unknown option ${arg}`);
if (argv.includes('--help')) { console.log('node scripts/configure-mcp.mjs --workspace DIRECTORY [--print] [--codex EXECUTABLE]\n--print emits configuration only; otherwise registers global Codex MCP servers.'); process.exit(0); }
const workspaceArg = option('--workspace');
if (!workspaceArg) throw Error('--workspace is required; use the same directory as the desktop app');
mkdirSync(resolve(workspaceArg), { recursive: true });
const workspace = realpathSync(resolve(workspaceArg));
const portable = existsSync(join(root, 'bin', 'spritelab'));
const engine = join(root, portable ? 'bin/spritelab' : 'target/debug/spritelab');
const server = join(root, 'adapters/mcp/server.mjs');
if (!existsSync(engine) || !existsSync(server) || !existsSync(join(root, 'adapters/mcp/node_modules/@modelcontextprotocol/sdk'))) throw Error('Build the engine and run npm ci --ignore-scripts in adapters/mcp first (portable bundles already include them).');
const definitions = Object.fromEntries(['live', 'headless'].map(mode => [`spritelab_${mode}`, { command: process.execPath, args: [server, '--engine', engine, '--workspace', workspace, ...(mode === 'live' ? ['--attach-latest'] : [])] }]));
if (argv.includes('--print')) console.log(JSON.stringify({ mcpServers: definitions }, null, 2));
else for (const [name, config] of Object.entries(definitions)) {
  const r = spawnSync(option('--codex') ?? 'codex', ['mcp', 'add', name, '--', config.command, ...config.args], { stdio: 'inherit' });
  if (r.error) throw r.error;
  if (r.status !== 0) process.exit(r.status ?? 1);
}
