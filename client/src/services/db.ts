import { AsyncInit } from '../utils/async-init';

const DB_MAIN = 'main';
const OS_AUTH = 'auth';
const KEY_REFRESH_TOKEN = 'refresh_token';

function requestAsPromise<T>(req: IDBRequest<T>): Promise<T> {
  return new Promise((accept, reject) => {
    req.onsuccess = () => accept(req.result);
    req.onerror = () => reject(req.error);
  });
}

export default new class {
  private db = new AsyncInit<IDBDatabase>();

  constructor() {
    const req = indexedDB.open(DB_MAIN, 0);
    req.onsuccess = () => this.db.set(req.result);
    req.onerror = () => { throw req.error; };
    req.onupgradeneeded = () => {
      req.result.createObjectStore(OS_AUTH)
    };
  }

  async getRefreshToken(): Promise<string | undefined> {
    const db = await this.db.get();
    const tx = db.transaction(OS_AUTH, 'readonly');
    return requestAsPromise(tx.objectStore(OS_AUTH).get(KEY_REFRESH_TOKEN));
  }

  async setRefreshToken(refreshToken: string): Promise<void> {
    const db = await this.db.get();
    const tx = db.transaction(OS_AUTH, 'readwrite');
    await requestAsPromise(tx.objectStore(OS_AUTH).put(refreshToken, KEY_REFRESH_TOKEN));
  }
}
