import type { Measurement } from "@/services/user";

interface StageDeleteMeasurementReq {
  measurementId: string;
}

interface StageUpdateMeasurementReq {
  measurement: Measurement;
}

export interface RequestMap {
  stageDeleteMeasurement: StageDeleteMeasurementReq;
  stageUpdateMeasurement: StageUpdateMeasurementReq;
}

export interface ResponseMap {
  stageDeleteMeasurement: void;
  stageUpdateMeasurement: string;
}

export type MessageType = keyof RequestMap & keyof ResponseMap;

export type RequestMessage<T = MessageType> = T extends MessageType
  ? {
    id: number;
    type: T;
    payload: RequestMap[T];
  }
  : never;

export type ResponseMessage<T = MessageType> = T extends MessageType
  ? { id: number; type: T } & (
    ResponseMap[T] extends void
      ? { payload?: ResponseMap[T] }
      : { payload: ResponseMap[T] }
    )
  : never;

declare const self: Window & { onconnect: (e: MessageEvent) => void };

self.onconnect = e => {
  const port = e.ports[0];

  port.onmessage = (e: MessageEvent<RequestMessage>) => {
    handle(e.data).then(r => port.postMessage(r));
  }
};

async function handle(req: RequestMessage): Promise<ResponseMessage> {
  switch (req.type) {
    case 'stageDeleteMeasurement': {
      return {
        id: req.id,
        type: req.type
      };
    }
    case 'stageUpdateMeasurement': {
      return {
        id: req.id,
        type: req.type,
        payload: 'done'
      };
    }
  }
}
