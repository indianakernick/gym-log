export interface RequestMessage {
  id: number;
  text: string;
  count: number;
}

export interface ResponseMessage {
  id: number;
  message: string;
}

declare const self: Window & { onconnect: (e: MessageEvent) => void };

self.onconnect = e => {
  const port = e.ports[0];

  port.onmessage = (e: MessageEvent<RequestMessage>) => {
    const msg: ResponseMessage = {
      id: e.data.id,
      message: e.data.text.repeat(e.data.count)
    };
    port.postMessage(msg);
  }
};
