declare module "@iarna/toml" {
  export class TomlError extends Error {}
  export function parse(input: string): Record<string, unknown>;
}
