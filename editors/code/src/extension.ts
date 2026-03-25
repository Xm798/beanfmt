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
  thousands?: string;
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
  if (typeof raw.thousands === "string") config.thousands = raw.thousands;
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

async function findProjectConfig(
  documentUri: vscode.Uri,
): Promise<BeanfmtConfig> {
  const workspaceFolder = vscode.workspace.getWorkspaceFolder(documentUri);
  if (!workspaceFolder) return {};

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
          return {};
        }
        // File not found — continue searching up
      }
    }
  }
  return {};
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

        // Helper: use explicit user setting if set, else project config, else default.
        // Uses config.inspect() to distinguish explicitly-set values from defaults.
        function resolve<T>(
          key: string,
          projectVal: T | undefined,
          fallback: T,
        ): T {
          const inspected = config.inspect<T>(key);
          // Explicit user setting at any level overrides project config
          const explicit =
            inspected?.workspaceFolderValue ??
            inspected?.workspaceValue ??
            inspected?.globalValue;
          if (explicit !== undefined) return explicit;
          if (projectVal !== undefined) return projectVal;
          return fallback;
        }

        // Normalize sort from config file (may be boolean)
        const projectSort =
          projectConfig.sort === true
            ? "asc"
            : projectConfig.sort === false
              ? "off"
              : (projectConfig.sort as string | undefined);

        const indent = clamp(
          resolve("indent", projectConfig.indent, 4),
          1,
          20,
        );
        const currencyColumn = clamp(
          resolve("currencyColumn", projectConfig.currency_column, 70),
          1,
          200,
        );
        const costColumn = clamp(
          resolve("costColumn", projectConfig.cost_column, 75),
          1,
          200,
        );
        const thousandsSeparator = resolve(
          "thousandsSeparator",
          projectConfig.thousands,
          "keep",
        );
        const spacesInBraces = resolve(
          "spacesInBraces",
          projectConfig.spaces_in_braces,
          false,
        );
        const fixedCJKWidth = resolve(
          "fixedCJKWidth",
          projectConfig.fixed_cjk_width,
          true,
        );
        const sort = resolve("sort", projectSort, "off");
        const sortTimeless = resolve(
          "sortTimeless",
          projectConfig.sort_timeless,
          "keep",
        );
        const sortExcludeRaw = resolve(
          "sortExclude",
          projectConfig.sort_exclude,
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
            thousandsSeparator,
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
