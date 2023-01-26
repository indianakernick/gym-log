// I was initially thinking that the service worker could manage the database
// and deal with syncing but then I wondered what sort of advantage that would
// have over doing it on the main thread. Passing a bunch of messages back and
// forth is going to have a performance impact. iOS doesn't implement periodic
// background sync so we could only sync while the app is in the foreground
// anyway.

export interface RequestMap {
  test: string;
}

export interface ResponseMap {
  test: number;
}

export type MessageType = keyof RequestMap & keyof ResponseMap;

type Message<T, P>
  = { id: number; type: T }
  & (
    P extends void
      ? { payload?: P }
      : { payload: P }
  );

export type RequestMessage<T = MessageType>
  = T extends MessageType ? Message<T, RequestMap[T]> : never;

export type ResponseMessage<T = MessageType>
  = T extends MessageType ? Message<T, ResponseMap[T]> : never;
