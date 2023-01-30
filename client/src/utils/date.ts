export function today(): string {
  return toString(shift(new Date()));
}

export function isValidDate(dateStr: string): boolean {
  const date = new Date(dateStr);
  return !Number.isNaN(date.valueOf()) && toString(date) === dateStr;
}

export function displayDate(dateStr: string): string {
  return formatter.format(shift(new Date(dateStr)));
}

const formatter = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium' });

function toString(date: Date): string {
  return date.toISOString().substring(0, 10);
}

function shift(date: Date): Date {
  return new Date(date.valueOf() - date.getTimezoneOffset() * 60000);
}
