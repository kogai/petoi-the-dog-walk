/**
 * E00: Send tokens to Bittle over a serial port and print every response line with a timestamp.
 * (experimentals/E00-connectivity.md)
 *
 * NOT YET RUN against a real Bittle. Usage (run by a human):
 *   node scripts/experiments/e00-serial-probe.ts <PORT> ksit kbalance khi
 *   node scripts/experiments/e00-serial-probe.ts --list
 * Each token is sent, then responses are collected for WAIT_MS (env E00_WAIT_MS, default 3000)
 * before the next one. Line ending is LF, or CRLF with E00_EOL=CRLF.
 * The stop token `d` is always sent at the end and on Ctrl-C.
 */
import { SerialPort } from "serialport";

const BAUD_RATE = 115200; // F-B4
const WAIT_MS = Number(process.env["E00_WAIT_MS"] ?? "3000");
const OPEN_SETTLE_MS = 2000; // the board may reset when the port opens
const STOP = "d"; // F-T1
const EOL = process.env["E00_EOL"] === "CRLF" ? "\r\n" : "\n";

const sleep = (ms: number): Promise<void> => new Promise((resolve) => setTimeout(resolve, ms));

const args = process.argv.slice(2);

if (args[0] === "--list") {
  const ports = await SerialPort.list();
  ports.forEach((p) => {
    console.log(p.path);
  });
  process.exit(0);
}

const [path, ...tokens] = args;
if (path === undefined || tokens.length === 0) {
  console.error("usage: node scripts/experiments/e00-serial-probe.ts <PORT> <token>...");
  process.exit(2);
}

const t0 = performance.now();
const stamp = (): string => `${(performance.now() - t0).toFixed(1).padStart(8)} ms`;

const port = new SerialPort({ path, baudRate: BAUD_RATE });
port.on("data", (chunk: Buffer) => {
  console.log(`${stamp()}  <- ${JSON.stringify(chunk.toString("latin1"))}`);
});
port.on("error", (err: Error) => {
  console.error(`${stamp()}  !! ${err.message}`);
});

const send = (token: string): Promise<void> =>
  new Promise((resolve, reject) => {
    console.log(`${stamp()}  -> ${JSON.stringify(token)}`);
    port.write(token + EOL, (err) => {
      if (err) reject(err);
      else
        port.drain(() => {
          resolve();
        });
    });
  });

const stopAndClose = async (): Promise<void> => {
  await send(STOP).catch(() => undefined);
  await sleep(500);
  port.close();
};

process.on("SIGINT", () => {
  void stopAndClose().then(() => process.exit(130));
});

await new Promise<void>((resolve, reject) => {
  port.on("open", () => {
    resolve();
  });
  port.on("error", reject);
});
await sleep(OPEN_SETTLE_MS);

await tokens.reduce(async (prev, token) => {
  await prev;
  await send(token);
  await sleep(WAIT_MS);
}, Promise.resolve());

await stopAndClose();
