// Tiny bridge test client: node bridge/ping.mjs [port] [command ...]
// Default: port 27029, commands PING, VERSION, REFS. Read-only unless you pass SENDJ/WRITE yourself.
import net from 'node:net';

const port = Number(process.argv[2]) || 27029;
const cmds = process.argv.length > 3 ? process.argv.slice(3) : ['PING', 'VERSION', 'REFS'];

const sock = net.connect(port, '127.0.0.1');
sock.setTimeout(15000, () => { console.error('timeout'); sock.destroy(); process.exit(1); });
sock.on('error', e => { console.error(`port ${port}: ${e.message}`); process.exit(1); });

let buf = '', i = 0;
sock.on('connect', () => sock.write(cmds[0] + '\n'));
sock.on('data', d => {
  buf += d;
  let nl;
  while ((nl = buf.indexOf('\n')) >= 0) {
    const line = buf.slice(0, nl); buf = buf.slice(nl + 1);
    console.log(`> ${cmds[i]}\n${line.split('\u001e').join('\n')}\n`);
    if (++i < cmds.length) sock.write(cmds[i] + '\n'); else sock.end();
  }
});
