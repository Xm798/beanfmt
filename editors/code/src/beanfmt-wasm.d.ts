// Auto-generated types from wasm-pack — keep in sync with wasm/beanfmt.d.ts
declare module "*/wasm" {
  export function format(
    input: string,
    indent: number,
    currency_column: number,
    cost_column: number,
    thousands: string,
    spaces_in_braces: boolean,
    fixed_cjk_width: boolean,
    sort: boolean,
  ): string;
  export function format_default(input: string): string;
}
