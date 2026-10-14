import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp, mkdir, writeFile, copyFile, readFile, rm, realpath} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {join, resolve} from 'node:path';
import {tmpdir} from 'node:os';
test('portable MCP registration discovers relocated paths and preserves argument boundaries', async () => {
  const root=await realpath(await mkdtemp(join(tmpdir(),'SpriteLab install with spaces ')));
  try {
    await mkdir(join(root,'scripts')); await mkdir(join(root,'bin'));
    await mkdir(join(root,'adapters/mcp/node_modules/@modelcontextprotocol/sdk'),{recursive:true});
    await copyFile(resolve('scripts/configure-mcp.mjs'),join(root,'scripts/configure-mcp.mjs'));
    await writeFile(join(root,'bin/spritelab'),''); await writeFile(join(root,'adapters/mcp/server.mjs'),'');
    const workspace=join(root,'my assets');
    const r=spawnSync(process.execPath,[join(root,'scripts/configure-mcp.mjs'),'--workspace',workspace,'--print'],{encoding:'utf8'});
    assert.equal(r.status,0,r.stderr);
    const config=JSON.parse(r.stdout).mcpServers;
    assert.equal(config.spritelab_live.command,process.execPath);
    assert.deepEqual(config.spritelab_live.args,[join(root,'adapters/mcp/server.mjs'),'--engine',join(root,'bin/spritelab'),'--workspace',workspace,'--attach-latest']);
    assert.deepEqual(config.spritelab_headless.args,config.spritelab_live.args.slice(0,-1));
    const mock=join(root,'mock-codex'); const recorded=join(root,'registered.jsonl');
    await writeFile(mock,`#!${process.execPath}\nrequire('node:fs').appendFileSync(${JSON.stringify(recorded)},JSON.stringify(process.argv.slice(2))+'\\n');\n`,{mode:0o700});
    const registered=spawnSync(process.execPath,[join(root,'scripts/configure-mcp.mjs'),'--workspace',workspace,'--codex',mock],{encoding:'utf8'});
    assert.equal(registered.status,0,registered.stderr);
    const calls=(await readFile(recorded,'utf8')).trim().split('\n').map(JSON.parse);
    assert.equal(calls.length,2);
    assert.deepEqual(calls[0],['mcp','add','spritelab_live','--',process.execPath,...config.spritelab_live.args]);
    assert.equal(spawnSync(process.execPath,[join(root,'scripts/configure-mcp.mjs'),'--print']).status,1);
    assert.equal(spawnSync(process.execPath,[join(root,'scripts/configure-mcp.mjs'),'--workspace',workspace,'--typo']).status,1);
  } finally {await rm(root,{recursive:true,force:true});}
});
