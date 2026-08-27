import * as vscode from "vscode";
import * as path from "path";
import * as TOML from "@iarna/toml";
import type * as BeanfmtWasm from "*/wasm";

let wasmModule: typeof BeanfmtWasm | undefined;

async function loadWasm(
  context: vscode.ExtensionContext,
): Promise<typeof BeanfmtWasm> {
  if (wasmModule) return wasmModule;
  const wasmPath = path.join(context.extensionPath, "wasm");
  // Use require() instead of import() because esbuild preserves native
  // import() which wraps CJS exports under .default in Node.js
  wasmModule = require(wasmPath) as typeof BeanfmtWasm;
  return wasmModule!;
}

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

interface BeanfmtConfig {
  indent?: number;
  currency_column?: number;
  cost_column?: number;
  inline_comment_column?: number;
  thousands?: string;
  decimal_mode?: string;
  decimal_places?: number;
  amount_scope?: string;
  spaces_in_braces?: boolean;
  fixed_cjk_width?: boolean;
  sort?: string | boolean;
  sort_timeless?: string;
  sort_exclude?: string[];
}

function validateConfig(raw: Record<string, unknown>): BeanfmtConfig {
  const config: BeanfmtConfig = {};
  if (typeof raw.indent === "number") config.indent = raw.indent;
  if (typeof raw.currency_column === "number")
    config.currency_column = raw.currency_column;
  if (typeof raw.cost_column === "number") config.cost_column = raw.cost_column;
  if (typeof raw.inline_comment_column === "number")
    config.inline_comment_column = raw.inline_comment_column;
  if (typeof raw.thousands === "string") config.thousands = raw.thousands;
  if (typeof raw.decimal_mode === "string")
    config.decimal_mode = raw.decimal_mode;
  if (typeof raw.decimal_places === "number")
    config.decimal_places = raw.decimal_places;
  if (typeof raw.amount_scope === "string")
    config.amount_scope = raw.amount_scope;
  if (typeof raw.spaces_in_braces === "boolean")
    config.spaces_in_braces = raw.spaces_in_braces;
  if (typeof raw.fixed_cjk_width === "boolean")
    config.fixed_cjk_width = raw.fixed_cjk_width;
  if (typeof raw.sort === "string" || typeof raw.sort === "boolean")
    config.sort = raw.sort;
  if (typeof raw.sort_timeless === "string")
    config.sort_timeless = raw.sort_timeless;
  if (Array.isArray(raw.sort_exclude))
    config.sort_exclude = raw.sort_exclude.filter(
      (s): s is string => typeof s === "string",
    );
  return config;
}

// Returns the parsed project config when a config file is found, or `null`
// when none exists. The distinction matters: a present config file is the
// single source of truth (editor settings are ignored), so callers must be
// able to tell "no file" apart from "file with no keys set".
async function findProjectConfig(
  documentUri: vscode.Uri,
): Promise<BeanfmtConfig | null> {
  const workspaceFolder = vscode.workspace.getWorkspaceFolder(documentUri);
  if (!workspaceFolder) return null;

  const fileDirParts = path.dirname(documentUri.fsPath).split(path.sep);
  const rootParts = workspaceFolder.uri.fsPath.split(path.sep);

  // Walk from file's directory up to workspace root (closest config wins)
  for (let i = fileDirParts.length; i >= rootParts.length; i--) {
    const dir = fileDirParts.slice(0, i).join(path.sep);
    for (const name of [".beanfmt.toml", "beanfmt.toml"]) {
      const configUri = vscode.Uri.file(path.join(dir, name));
      try {
        const content = await vscode.workspace.fs.readFile(configUri);
        const raw = TOML.parse(new TextDecoder().decode(content));
        return validateConfig(raw as Record<string, unknown>);
      } catch (err: unknown) {
        if (err instanceof Error && err.name === "TomlError") {
          vscode.window.showWarningMessage(
            `[beanfmt] Failed to parse ${name}: ${err.message}`,
          );
          // A file exists but is unusable — fall back to editor settings
          // rather than silently formatting with built-in defaults.
          return null;
        }
        // File not found — continue searching up
      }
    }
  }
  return null;
}

export async function activate(
  context: vscode.ExtensionContext,
): Promise<void> {
  console.log("[beanfmt] activated");

  // Preload WASM module to avoid blocking extension host on first format
  loadWasm(context).catch((err) => {
    console.error("[beanfmt] failed to preload WASM:", err);
  });

  const disposable = vscode.languages.registerDocumentFormattingEditProvider(
    { scheme: "file", language: "beancount" },
    {
      async provideDocumentFormattingEdits(
        document: vscode.TextDocument,
      ): Promise<vscode.TextEdit[]> {
        const projectConfig = await findProjectConfig(document.uri);
        const config = vscode.workspace.getConfiguration("beanfmt");

        // Prettier model: a project config file is the single source of truth.
        // When `.beanfmt.toml` is present, editor settings are ignored entirely
        // (even for keys it omits, which fall back to built-in defaults) so that
        // format-on-save matches the `beanfmt` CLI / CI exactly. Only when no
        // config file exists do explicit editor settings act as the config.
        const pc = projectConfig ?? {};

        function resolve<T>(
          key: string,
          projectVal: T | undefined,
          fallback: T,
        ): T {
          if (projectConfig !== null) {
            return projectVal !== undefined ? projectVal : fallback;
          }
          // No config file — use explicit editor setting, else default.
          // config.inspect() distinguishes explicitly-set values from defaults.
          const inspected = config.inspect<T>(key);
          const explicit =
            inspected?.workspaceFolderValue ??
            inspected?.workspaceValue ??
            inspected?.globalValue;
          if (explicit !== undefined) return explicit;
          return fallback;
        }

        // Normalize sort from config file (may be boolean)
        const projectSort =
          pc.sort === true
            ? "asc"
            : pc.sort === false
              ? "off"
              : (pc.sort as string | undefined);

        const indent = clamp(resolve("indent", pc.indent, 2), 1, 20);
        const currencyColumn = clamp(
          resolve("currencyColumn", pc.currency_column, 70),
          1,
          200,
        );
        const costColumn = clamp(
          resolve("costColumn", pc.cost_column, 75),
          1,
          200,
        );
        const inlineCommentColumn = clamp(
          resolve("inlineCommentColumn", pc.inline_comment_column, 0),
          0,
          200,
        );
        const thousandsSeparator = resolve(
          "thousandsSeparator",
          pc.thousands,
          "keep",
        );
        const decimalMode = resolve("decimalMode", pc.decimal_mode, "keep");
        const decimalPlaces = clamp(
          resolve("decimalPlaces", pc.decimal_places, 2),
          0,
          20,
        );
        const amountScope = resolve("amountScope", pc.amount_scope, "all");
        const spacesInBraces = resolve(
          "spacesInBraces",
          pc.spaces_in_braces,
          false,
        );
        const fixedCJKWidth = resolve("fixedCJKWidth", pc.fixed_cjk_width, true);
        const sort = resolve("sort", projectSort, "off");
        const sortTimeless = resolve("sortTimeless", pc.sort_timeless, "keep");
        const sortExcludeRaw = resolve(
          "sortExclude",
          pc.sort_exclude,
          [] as string[],
        );
        const sortExclude =
          sortExcludeRaw.length > 0 ? sortExcludeRaw : undefined;

        const input = document.getText().replace(/\r\n?/g, "\n");

        try {
          const wasm = await loadWasm(context);
          const result = wasm.format(
            input,
            indent,
            currencyColumn,
            costColumn,
            inlineCommentColumn,
            thousandsSeparator,
            decimalMode,
            decimalPlaces,
            amountScope,
            spacesInBraces,
            fixedCJKWidth,
            sort,
            sortTimeless,
            sortExclude,
          );

          if (result === input) {
            return [];
          }

          const fullRange = new vscode.Range(
            document.lineAt(0).range.start,
            document.lineAt(document.lineCount - 1).range.end,
          );
          return [vscode.TextEdit.replace(fullRange, result)];
        } catch (err) {
          const message = err instanceof Error ? err.message : String(err);
          vscode.window.showErrorMessage(`Beanfmt error: ${message}`);
          return [];
        }
      },
    },
  );

  context.subscriptions.push(disposable);
}

export function deactivate(): void {
  console.log("[beanfmt] deactivated");
}
