const formatter = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium' });

export function displayDate(date: string): string {
  return formatter.format(new Date(date));
}

export function toDbDate(date: Date): string {
  return date.toISOString().substring(0, 10);
}

export function fromDbDate(date: string): Date {
  return new Date(date);
}
