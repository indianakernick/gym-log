export class AsyncInit<T> {
  private value: T | undefined;
  private readonly waiters: ((_: T) => void)[] = [];

  constructor(value?: T | undefined) {
    this.value = value;
  }

  get initialised(): boolean {
    return !!this.value;
  }

  get(): Promise<T> {
    return new Promise(accept => {
      if (this.value) {
        accept(this.value);
      } else {
        this.waiters.push(accept);
      }
    });
  }

  set(value: T) {
    this.value = value;
    while (true) {
      const waiter = this.waiters.shift();
      if (!waiter) break;
      waiter(value);
    }
  }
}
