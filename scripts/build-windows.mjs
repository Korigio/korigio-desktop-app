#!/usr/bin/env node
/**
 * Windows-only NSIS production build.
 * Delegates to scripts/build-windows.ps1 (Tauri signs when a thumbprint is configured).
 */
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

if (process.platform !== "win32") {
  console.error(
    "npm run build:windows must run on Windows. On macOS use GitHub Actions (windows-latest) or a Windows VM.",
  );
  process.exit(1);
}

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const script = join(root, "scripts", "build-windows.ps1");
const shell = process.env.ComSpec ? "powershell.exe" : "pwsh";
const result = spawnSync(
  shell,
  ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script],
  { stdio: "inherit", cwd: root, windowsHide: true },
);

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);
