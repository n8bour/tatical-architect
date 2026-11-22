// Dev harness: runs the app's real FMB code (src/ui/fmbridge.js) from Node, talking to the plugin over TCP.
//   import { FMB, FMB_IDS } from './fmb-node.mjs'   (port: env FMB_PORT, default 27029)
// Read-only helpers are safe; applyXI / applySetPieces / setTakers WRITE to the running game.
import net from 'node:net';
import fs from 'node:fs';
import vm from 'node:vm';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const PORT = Number(process.env.FMB_PORT) || 27029;
const here = path.dirname(fileURLToPath(import.meta.url));

function bridgeCall(cmd, timeoutMs = 15000) {
  return new Promise((resolve, reject) => {
    const s = net.connect(PORT, '127.0.0.1');
    let buf = '';
    s.setTimeout(timeoutMs, () => { s.destroy(); reject(new Error('timeout: ' + cmd.slice(0, 60))); });
    s.on('error', reject);
    s.on('connect', () => s.write(cmd + '\n'));
    s.on('data', d => { buf += d; const nl = buf.indexOf('\n'); if (nl >= 0) { s.end(); resolve(buf.slice(0, nl)); } });
  });
}

const ctx = vm.createContext({
  DESKTOP: true, console, setTimeout, JSON, Promise, Date, Math, Number, String, Object, Array, Set, Error,
  window: { __TAURI__: { core: { invoke: async (_, a) => ({ port: PORT, reply: await bridgeCall(a.cmd, a.timeoutMs) }) } } },
  L: (pt, en) => en,
});
if (process.env.FMB_CORE) for (const f of ['i18n/i18n.js', 'core/roles.js', 'core/tactics_data.js', 'core/engine.js']) vm.runInContext(fs.readFileSync(path.join(here, '../src', f), 'utf8') + ';globalThis.ROLE_BY_ID = typeof ROLE_BY_ID !== "undefined" ? ROLE_BY_ID : globalThis.ROLE_BY_ID;', ctx);
vm.runInContext(fs.readFileSync(path.join(here, '../src/ui/fmbridge.js'), 'utf8') + '\n;globalThis.FMB = FMB; globalThis.FMB_IDS = FMB_IDS;', ctx);

export const FMB = ctx.FMB, FMB_IDS = ctx.FMB_IDS, call = bridgeCall, sandbox = ctx, evalIn = (code) => vm.runInContext(code, ctx);
