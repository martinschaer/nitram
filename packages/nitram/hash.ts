export function objectHash(obj: Record<string, unknown>): number {
  const str = deterministicStringify(obj);
  return hashString(str);
}

function deterministicStringify(obj: unknown): string {
  if (obj === null || typeof obj !== "object") {
    return JSON.stringify(obj);
  }
  if (Array.isArray(obj)) {
    return `[${obj.map(deterministicStringify).join(",")}]`;
  }
  // Sort keys to ensure consistent ordering
  const sortedKeys = Object.keys(obj).sort();
  const keyValuePairs = sortedKeys.map(
    (key) => `${JSON.stringify(key)}:${deterministicStringify(obj[key])}`,
  );
  return `{${keyValuePairs.join(",")}}`;
}

function hashString(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = (hash << 5) - hash + char;
    hash |= 0; // Convert to 32-bit integer
  }
  return hash;
}
