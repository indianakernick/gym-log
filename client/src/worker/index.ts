import type { RequestMessage, ResponseMessage } from '@/model/worker';

declare const self: Window & { onconnect: (e: MessageEvent) => void };

self.onconnect = e => {
  const port = e.ports[0];

  port.onmessage = (e: MessageEvent<RequestMessage>) => {
    handle(e.data).then(r => port.postMessage(r));
  }
};

async function handle(req: RequestMessage): Promise<ResponseMessage> {
  switch (req.type) {
    case 'test':
      return {
        id: req.id,
        type: req.type,
        payload: req.payload.length
      };
  }
}
