export function itemLines<T>(groups: T[][], groupIdx: number, itemIdx: number) {
  if (itemIdx === groups[groupIdx].length - 1) {
    return groupIdx === groups.length - 1 ? 'full' : 'none';
  }
  return 'inset';
}
