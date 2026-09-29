export function generateSuffix(): string {
  return Date.now().toString(36) + Math.random().toString(36).slice(2, 6);
}

export function uniqueTitle(prefix = 'Item'): string {
  return `${prefix}_${generateSuffix()}`;
}

export function uniqueUsername(prefix = 'user'): string {
  return `${prefix}_${generateSuffix()}`;
}
