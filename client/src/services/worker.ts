import type {
  MessageType,
  RequestMap,
  RequestMessage,
  ResponseMap,
  ResponseMessage
} from "../worker";

export default new class {
  private readonly worker: SharedWorker;
  private readonly handlers = new Map<number, (_: ResponseMessage) => void>();
  private id: number = 0;

  constructor() {
    this.worker = new SharedWorker(new URL('../worker/index.ts', import.meta.url));
    this.worker.port.onmessage = this.onMessage.bind(this);
  }

  sendRequest<T extends MessageType>(
    type: T,
    payload: RequestMap[T]
  ): Promise<ResponseMap[T]> {
    const id = this.id++;
    this.worker.port.postMessage({ id, type, payload } as RequestMessage);
    return new Promise(accept => {
      this.handlers.set(id, res => accept(res.payload as ResponseMap[T]));
    });
  }

  private onMessage(event: MessageEvent<ResponseMessage>) {
    const id = event.data.id;
    const handler = this.handlers.get(id);
    if (handler) handler(event.data);
    this.handlers.delete(id);
  }
}
