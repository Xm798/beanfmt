/* tslint:disable */
/* eslint-disable */

/**
 * Format a beancount document with full options.
 */
export function format(input: string, indent: number, currency_column: number, cost_column: number, thousands: string, spaces_in_braces: boolean, fixed_cjk_width: boolean, sort: boolean): string;

/**
 * Format with default options (convenience function).
 */
export function format_default(input: string): string;
