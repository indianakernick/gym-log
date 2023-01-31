export function toDateString(date: Date): string {
  return toString(shift(date));
}

export function isValidDate(dateStr: string): boolean {
  const date = new Date(dateStr);
  return !Number.isNaN(date.valueOf()) && toString(date) === dateStr;
}

export function displayDate(dateStr: string): string {
  return dateFormatter.format(shift(new Date(dateStr)));
}

export function displayDateTime(dateTimeStr: string): string {
  return dateTimeFormatter.format(new Date(dateTimeStr));
}

const dateFormatter = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium' });
const dateTimeFormatter = new Intl.DateTimeFormat(undefined, {
  dateStyle: 'short',
  timeStyle: 'short'
});

function toString(date: Date): string {
  return date.toISOString().substring(0, 10);
}

function shift(date: Date): Date {
  return new Date(date.valueOf() - date.getTimezoneOffset() * 60000);
}
