export function binarySearch<T>(
  array: T[],
  start: number,
  end: number,
  compare: (element: T) => number,
): number {
  while (start < end) {
    const middle = start + ((end - start) >> 1);
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

export function groupBy<T, U>(
  array: T[],
  getKey: (element: T) => U,
): T[][] {
  const groups: T[][] = [];

  if (array.length > 0) {
    let previous = getKey(array[0]);

    for (let i = 1; i < array.length; ++i) {
      const current = getKey(array[i]);

      if (current !== previous) {
        groups.push(array.splice(0, i));
        previous = current;
        i = 0;
      }
    }

    groups.push(array);
  }

  return groups;
}

export function groupByFiltered<T, U>(
  array: T[],
  getKey: (element: T) => U | undefined,
): T[][] {
  const groups: T[][] = [];

  if (array.length > 0) {
    let previous = getKey(array[0]);

    while (previous === undefined) {
      array.splice(0, 1);
      previous = getKey(array[0]);
    }

    if (array.length === 0) return [];

    for (let i = 1; i < array.length; ++i) {
      const current = getKey(array[i]);

      if (current === undefined) {
        array.splice(i, 1);
        --i;
      } else if (current !== previous) {
        groups.push(array.splice(0, i));
        previous = current;
        i = 0;
      }
    }

    if (array.length) groups.push(array);
  }

  return groups;
}
