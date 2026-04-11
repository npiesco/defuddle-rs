/**
 * Candycam recorder client.
 *
 * Copied from the Duckcells demo pattern and kept intentionally close so
 * the fsgdb demo can switch recording between terminal, VS Code, and agent
 * windows by PID with minimal new logic.
 */
import { spawn, type ChildProcess } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const VENV_PYTHON = path.join(__dirname, '.venv', 'Scripts', 'python.exe');
const HELPER_SCRIPT = path.join(__dirname, 'candycam_helper.py');

export class CandycamClient {
  private child: ChildProcess | null = null;
  private buffer = '';
  readonly segmentPaths: string[] = [];
  private enabled: boolean;

  constructor() {
    this.enabled = process.env.DEMO_NO_RECORD !== '1' && process.env.DEMO_DRY_RUN !== '1';
  }

  async start(): Promise<void> {
    if (!this.enabled) return;

    this.child = spawn(VENV_PYTHON, [HELPER_SCRIPT], {
      env: { ...process.env, CANDYCAM_BACKEND: process.env.CANDYCAM_BACKEND ?? 'xcap' },
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    this.child.stderr?.on('data', (chunk: Buffer) => {
      const line = chunk.toString().trim();
      if (line) console.log(`[candycam] ${line}`);
    });

    await this.waitFor('READY');
  }

  async recordWindow(filename: string, titleSubstring: string): Promise<void> {
    if (!this.enabled) return;
    const outputPath = path.join(__dirname, 'output', filename);
    this.send(`RECORD_WINDOW ${outputPath} ${titleSubstring}`);
    const line = await this.waitFor('RECORDING', 'ERROR');
    if (line.startsWith('ERROR')) {
      throw new Error(`candycam: failed to record window "${titleSubstring}": ${line}`);
    }
    this.segmentPaths.push(outputPath);
  }

  async recordWindowByPID(filename: string, pid: number): Promise<void> {
    if (!this.enabled) return;
    const outputPath = path.join(__dirname, 'output', filename);
    this.send(`RECORD_WINDOW_PID ${outputPath} ${pid}`);
    const line = await this.waitFor('RECORDING', 'ERROR');
    if (line.startsWith('ERROR')) {
      throw new Error(`candycam: failed to record window for PID ${pid}: ${line}`);
    }
    this.segmentPaths.push(outputPath);
  }

  async stopSegment(): Promise<void> {
    if (!this.enabled) return;
    this.send('STOP');
    const line = await this.waitFor('STOPPED', 'ERROR');
    if (line.startsWith('ERROR')) {
      throw new Error(`candycam: segment corrupt after STOP: ${line}`);
    }
  }

  async quit(): Promise<void> {
    if (!this.enabled || !this.child) return;
    this.send('QUIT');
    await this.waitFor('QUIT');
    this.child.stdin?.end();
    this.child = null;
  }

  private send(command: string): void {
    this.child?.stdin?.write(command + '\n');
  }

  private waitFor(...prefixes: string[]): Promise<string> {
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error(`candycam: timed out waiting for ${prefixes.join('|')}`));
      }, 15_000);

      const check = () => {
        const lines = this.buffer.split('\n');
        for (let i = 0; i < lines.length; i++) {
          for (const prefix of prefixes) {
            if (lines[i].startsWith(prefix)) {
              const match = lines[i];
              this.buffer = lines.slice(i + 1).join('\n');
              clearTimeout(timeout);
              resolve(match);
              return true;
            }
          }
        }
        return false;
      };

      if (check()) return;

      const onData = (chunk: Buffer) => {
        this.buffer += chunk.toString();
        if (check()) {
          this.child?.stdout?.removeListener('data', onData);
        }
      };
      this.child?.stdout?.on('data', onData);
    });
  }
}
