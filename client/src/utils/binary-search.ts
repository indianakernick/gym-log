export function binarySearch<T>(
  array: T[],
  start: number,
  end: number,
  compare: (element: T) => number
): number {
  while (start < end) {
    const middle = (start + (end - start) / 2) | 0;
    const diff = compare(array[middle])

    if (diff < 0) {
      start = middle + 1;
    } else if (diff > 0) {
      end = middle;
    } else {
      return middle;
    }
  }

  return -1;
}

// Much faster than localeCompare
export function stringCompare(a: string, b: string): number {
  return a < b ? -1 : (a > b ? 1 : 0);
}
