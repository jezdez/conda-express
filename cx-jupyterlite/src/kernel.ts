import { WebWorkerKernel } from '@jupyterlite/xeus';
import type { KernelMessage } from '@jupyterlab/services';

const CONDA_RE = /^([%!]?)conda\s+(\w+)/gm;

/**
 * Kernel subclass that rewrites `%conda` / `!conda` / bare `conda`
 * commands to `%cx` / `!cx` / `cx` before the message reaches the
 * WebWorker. This prevents mambajs's `processMagics` from intercepting
 * the command, letting it pass through to Python where real conda
 * (via conda-emscripten and cx-wasm) handles it.
 */
export class CxWebWorkerKernel extends WebWorkerKernel {
  async handleMessage(msg: KernelMessage.IMessage): Promise<void> {
    if (msg.header.msg_type === 'execute_request') {
      const content = msg.content as KernelMessage.IExecuteRequestMsg['content'];
      content.code = content.code.replace(CONDA_RE, '$1cx $2');
    }
    await super.handleMessage(msg);
  }
}
