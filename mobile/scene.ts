// Test scenes (#1026): `#scene=<id>[&lang=en|ru][&theme=plum|brass|felt][&speed=1|2|4]`
// opens one screen on a scenes server (server/src/mvp/scenes.ts). This module
// is main.ts's first import: on a scene link it swaps localStorage for one in
// memory before api.ts, lang.ts or theme.ts read it, so a scene's fixture
// player, language and palette never touch the browser's saved ones.
import { CG } from "./cg";

const params = new URLSearchParams(location.hash.slice(1));

/** The scene's id, or null on any other address. */
export const sceneId: string | null = CG ? null : params.get("scene");

class MemoryStorage implements Storage {
  private m = new Map<string, string>();
  get length() {
    return this.m.size;
  }
  clear(): void {
    this.m.clear();
  }
  getItem(k: string): string | null {
    return this.m.get(k) ?? null;
  }
  key(i: number): string | null {
    return [...this.m.keys()][i] ?? null;
  }
  removeItem(k: string): void {
    this.m.delete(k);
  }
  setItem(k: string, v: string): void {
    this.m.set(k, String(v));
  }
}

if (sceneId) {
  const mem = new MemoryStorage();
  const lang = params.get("lang");
  const theme = params.get("theme");
  const speed = params.get("speed");
  if (lang) mem.setItem("arena.lang", lang);
  if (theme) mem.setItem("arena.theme", theme);
  if (speed) mem.setItem("arena.battleSpeed", speed);
  Object.defineProperty(window, "localStorage", { value: mem, configurable: true });
}
