export function readCssVar(name: string): string {
  const root = getComputedStyle(document.documentElement);
  const direct = root.getPropertyValue(name).trim();
  if (direct && !direct.startsWith("var(")) {
    return direct;
  }
  if (name.startsWith("--color-")) {
    const alias = `--servioo-${name.slice("--color-".length)}`;
    return root.getPropertyValue(alias).trim();
  }
  return direct;
}
