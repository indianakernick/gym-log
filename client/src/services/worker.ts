import type { RequestMessage, ResponseMessage } from "../worker";

export default new class {
  private readonly worker: SharedWorker;
  private readonly handlers = new Map<number, (_: ResponseMessage) => void>();
  private id: number = 0;

  constructor() {
    this.worker = new SharedWorker(new URL('../worker/index.ts', import.meta.url));
    this.worker.port.onmessage = this.onMessage.bind(this);
  }

  sendRequest(text: string, count: number): Promise<string> {
    const id = this.id++;
    const msg: RequestMessage = { id, text, count };
    this.worker.port.postMessage(msg);
    return new Promise(accept => {
      this.handlers.set(id, msg => accept(msg.message));
    });
  }

  private onMessage(event: MessageEvent<ResponseMessage>) {
    const id = event.data.id;
    const handler = this.handlers.get(id);
    if (handler) handler(event.data);
    this.handlers.delete(id);
  }
}
