export function interpolate(
  template: string,
  vars: Record<string, string | undefined>
): string {
  return template.replace(/\$\{([^}]+)\}/g, (_match, expr) => {
    const colonIndex = expr.indexOf(':-');
    
    if (colonIndex !== -1) {
      const varName = expr.substring(0, colonIndex).trim();
      const defaultValue = expr.substring(colonIndex + 2).trim();
      return vars[varName] ?? defaultValue;
    }
    
    return vars[expr.trim()] ?? '';
  });
}

export function interpolateRecord(
  record: Record<string, string>,
  vars: Record<string, string | undefined>
): Record<string, string> {
  const result: Record<string, string> = {};
  for (const [key, value] of Object.entries(record)) {
    result[key] = interpolate(value, vars);
  }
  return result;
}