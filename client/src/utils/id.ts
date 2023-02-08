let count = 0;

export function getIdGenerator(prefix: string) {
  const id = `${prefix}-${count++}`;
  return (suffix?: string) => {
    return suffix ? `${id}-${suffix}` : id;
  };
}
