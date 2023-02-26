import type { Deleted } from '@/model/db';

export function colorForChange<T extends object, U>(
  entity: T,
  otherEntity: T | Deleted,
  access: (e: T) => U | undefined
): { [key in string]: boolean } {
  if ('deleted' in otherEntity) return {};
  const value = access(entity);
  const otherValue = access(otherEntity);
  return {
    'text-green-400': otherValue === undefined,
    'text-orange-400': otherValue !== undefined && value !== otherValue
  };
}
