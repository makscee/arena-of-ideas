// One browser-discovery seam for every e2e probe and capture walk. Prefer the
// Playwright-managed Chromium when installed, then fall back to a system Chrome
// without requiring a developer-specific path in source.

import { accessSync, constants, existsSync } from "node:fs";
import { delimiter } from "node:path";
import { chromium } from "playwright";

function executable(path) {
  if (!path || !existsSync(path)) return false;
  try {
    accessSync(path, process.platform === "win32" ? constants.F_OK : constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

function pathCandidates(env) {
  const names = process.platform === "win32"
    ? ["chrome.exe", "msedge.exe"]
    : ["google-chrome", "google-chrome-stable", "chromium", "chromium-browser", "microsoft-edge"];
  return (env.PATH ?? "").split(delimiter).flatMap((dir) => names.map((name) => `${dir}/${name}`));
}

export function discoverBrowser(env = process.env) {
  if (env.AOI_E2E_FORCE_NO_BROWSER === "1") return null;

  const explicit = env.AOI_E2E_BROWSER_PATH;
  if (explicit) return executable(explicit) ? { kind: "explicit", executablePath: explicit } : null;

  const bundled = chromium.executablePath();
  if (executable(bundled)) return { kind: "playwright", executablePath: bundled };

  const platformCandidates = process.platform === "darwin"
    ? [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
      ]
    : process.platform === "win32"
      ? [
          `${env.PROGRAMFILES ?? ""}/Google/Chrome/Application/chrome.exe`,
          `${env["PROGRAMFILES(X86)"] ?? ""}/Google/Chrome/Application/chrome.exe`,
          `${env.LOCALAPPDATA ?? ""}/Google/Chrome/Application/chrome.exe`,
        ]
      : ["/usr/bin/google-chrome", "/usr/bin/google-chrome-stable", "/usr/bin/chromium", "/usr/bin/chromium-browser"];

  const system = [...platformCandidates, ...pathCandidates(env)].find(executable);
  return system ? { kind: "system", executablePath: system } : null;
}

export const noBrowserMessage =
  "e2e: no Chromium browser found; install Playwright Chromium (`npx playwright install chromium`), install system Chrome/Chromium, or set AOI_E2E_BROWSER_PATH";

export function requireBrowser() {
  const found = discoverBrowser();
  if (!found) throw new Error(noBrowserMessage);
  return found;
}

export async function launchChromium(options = {}) {
  const found = requireBrowser();
  return chromium.launch({ ...options, executablePath: found.executablePath });
}
