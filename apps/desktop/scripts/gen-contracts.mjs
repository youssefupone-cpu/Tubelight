// gen-contracts.mjs — regenerates apps/desktop/src/bindings.ts from Rust via tauri-specta.
//
// In debug builds, yoube_desktop_lib::run() calls Builder::export(...) which writes the typed
// bindings to src/bindings.ts before the window opens. This script launches `tauri dev` (which
// builds & runs the Rust binary in debug mode → triggers the export), waits just long enough for
// the export to land, then stops the dev server.
//
// Requirement: the Tauri Linux dev stack must be installed on the machine
// (libwebkit2gtk-4.1-dev libgtk-3-dev libglib2.0-dev libssl-dev pkg-config + deps). In headless
// environments without those libs, the committed src/bindings.ts stub is used instead (see its
// header) and this script exits cleanly without failing the workspace.
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const desktopDir = resolve(__dirname, "..");
const timeoutMs = Number(process.env.GEN_CONTRACTS_TIMEOUT_MS ?? 25_000);

const child = spawn("pnpm", ["exec", "tauri", "dev"], {
  cwd: desktopDir,
  stdio: "inherit",
  shell: process.platform === "win32",
});

const timer = setTimeout(() => {
  child.kill("SIGTERM");
}, timeoutMs);

child.on("error", (err) => {
  clearTimeout(timer);
  console.error("[gen-contracts] failed to spawn tauri:", err.message);
  process.exit(0);
});

child.on("exit", (code, signal) => {
  clearTimeout(timer);
  const msg = signal === "SIGTERM" ? "timeout-reached (bindings exported on startup)" : `exit ${code}`;
  console.log(`[gen-contracts] tauri dev stopped: ${msg}`);
  process.exit(0);
});

process.on("SIGINT", () => child.kill("SIGTERM"));
process.on("SIGTERM", () => child.kill("SIGTERM"));
