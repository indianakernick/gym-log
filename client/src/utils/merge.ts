import type { Deleted } from '@/model/db';

export type ChangeDesc = 'equal' | 'modified' | 'added';

export function describeChange<T extends object, U>(
  entity: T,
  otherEntity: T | Deleted,
  access: (e: T) => U | undefined
): ChangeDesc {
  if ('deleted' in otherEntity) return 'equal';
  const value = access(entity);
  const otherValue = access(otherEntity);
  if (otherValue === undefined) return 'added';
  if (value !== otherValue) return 'modified';
  return 'equal';
}
