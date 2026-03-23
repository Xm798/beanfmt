import * as vscode from "vscode";
import * as path from "path";
import type * as BeanfmtWasm from "*/wasm";

let wasmModule: typeof BeanfmtWasm | undefined;

async function loadWasm(
  context: vscode.ExtensionContext,
): Promise<typeof BeanfmtWasm> {
  if (wasmModule) return wasmModule;
  const wasmPath = path.join(context.extensionPath, "wasm");
  wasmModule = await import(wasmPath);
  return wasmModule!;
}

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
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
        const config = vscode.workspace.getConfiguration("beanfmt");
        const indent = config.get<string>("indent", "    ").slice(0, 20);
        const currencyColumn = clamp(
          config.get<number>("currencyColumn", 70),
          1,
          200,
        );
        const costColumn = clamp(
          config.get<number>("costColumn", 75),
          1,
          200,
        );
        const thousandsSeparator = config.get<string>(
          "thousandsSeparator",
          "keep",
        );
        const spacesInBraces = config.get<boolean>("spacesInBraces", false);
        const fixedCJKWidth = config.get<boolean>("fixedCJKWidth", true);
        const sort = config.get<boolean>("sort", false);

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
