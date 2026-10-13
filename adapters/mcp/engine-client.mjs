import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { randomUUID } from 'node:crypto';

/** An adapter to one Rust session. It contains no editing rules. */
export class EngineClient {
  #child; #pending = new Map(); #closed = false;
  constructor(binary, workspace, socket) {
    this.#child = spawn(binary, socket ? ['--attach', socket] : ['--workspace', workspace], { stdio: ['pipe', 'pipe', 'inherit'] });
    const lines = createInterface({ input: this.#child.stdout });
    lines.on('line', line => {
      try {
        const response = JSON.parse(line);
        const entry = this.#pending.get(response.request_id);
        if (!entry) { this.#fail(new Error('Unexpected engine response')); return; }
        this.#pending.delete(response.request_id); clearTimeout(entry.timer); entry.resolve(response);
      } catch (error) { this.#fail(error); }
    });
    this.#child.on('error', error => this.#fail(error));
    this.#child.on('exit', (code, signal) => this.#fail(new Error(`Engine session ended (${code ?? signal})`)));
    this.#child.stdin.on('error', error => this.#fail(error));
  }
  execute(request) {
    if (this.#closed) return Promise.reject(new Error('Engine session is closed'));
    if (this.#pending.size >= 64) return Promise.reject(new Error('Adapter queue limit exceeded'));
    const request_id = request.request_id ?? randomUUID();
    if (this.#pending.has(request_id)) return Promise.reject(new Error('Duplicate in-flight request_id'));
    const data = JSON.stringify({ ...request, request_id }) + '\n';
    if (Buffer.byteLength(data) > 16 * 1024 * 1024) return Promise.reject(new Error('Request exceeds 16 MiB'));
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => this.#fail(new Error('Engine timed out; session closed; do not assume rollback')), 30_000);
      this.#pending.set(request_id, { resolve, reject, timer });
      this.#child.stdin.write(data);
    });
  }
  #fail(error) {
    this.#closed = true;
    for (const entry of this.#pending.values()) { clearTimeout(entry.timer); entry.reject(error); }
    this.#pending.clear(); this.#child.kill();
  }
  close() { this.#fail(new Error('Engine client closed')); }
}
