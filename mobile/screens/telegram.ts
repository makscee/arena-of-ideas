// M4-6 (mission #810): log in with Telegram. The sheet makes a one-time code,
// opens the bot (a phone opens the Telegram app; a laptop also shows a QR to
// scan), and polls until the bot accepts it: then this device is that player.
// "Link Telegram" (the title menu) is the same sheet for a player who is in.
import qrcode from "qrcode-generator";
import { api, ApiError } from "../api";
import { button, closable, h } from "../ui/dom";

/** How often the sheet asks whether the bot accepted. */
const POLL_MS = 2_000;

/** The Log in / Link Telegram sheet. `fake`: the dev server's fake bot, which
 * a button plays. `onDone` runs once this device is the player. */
export function telegramSheet(opts: { link: boolean; fake: boolean; onDone: () => void }): void {
  const status = h("p", { class: "dim small", "data-testid": "tg-status" }, "…");
  const body = h("div", { class: "stack", "data-testid": "tg-sheet" }, h("h2", {}, opts.link ? "Link Telegram" : "Log in with Telegram"), status);
  const close = closable(body);
  void begin();

  async function begin(): Promise<void> {
    body.querySelectorAll(".tg-step").forEach((el) => el.remove());
    status.textContent = "…";
    let start;
    try {
      start = await api.telegramStart();
    } catch (e) {
      status.textContent = e instanceof Error ? e.message : String(e);
      return;
    }
    const open = h("a", { class: "button primary tg-step", href: start.url, target: "_blank", rel: "noopener", "data-testid": "tg-open" }, "Open Telegram");
    status.textContent = "Press Start in the bot, then come back: this page logs you in by itself.";
    body.append(open);
    // A laptop: scan it with the phone that has Telegram.
    if (matchMedia("(hover: hover) and (pointer: fine)").matches) {
      const qr = qrcode(0, "M");
      qr.addData(start.url);
      qr.make();
      body.append(h("img", { class: "tg-qr tg-step", src: qr.createDataURL(5, 2), alt: "QR code: scan it with your phone's camera", "data-testid": "tg-qr" }));
    }
    if (opts.fake) {
      const code = start.code;
      body.append(button("Dev: the bot accepts", () => void api.devTelegramAccept(code).catch((e) => (status.textContent = String(e))), "small tg-step", "tg-fake-accept"));
    }
    void poll(start.code, Date.parse(start.expiresAt));
  }

  async function poll(code: string, until: number): Promise<void> {
    while (body.isConnected && Date.now() < until) {
      await new Promise((r) => setTimeout(r, POLL_MS));
      if (!body.isConnected) return;
      try {
        if ((await api.telegramPoll(code)).status === "done") {
          close();
          return opts.onDone();
        }
      } catch (e) {
        if (!(e instanceof ApiError && e.status === 404)) continue; // a blip: ask again
        break;
      }
    }
    if (!body.isConnected) return;
    body.querySelectorAll(".tg-step").forEach((el) => el.remove());
    status.textContent = "This login has expired.";
    body.append(button("Try again", () => void begin(), "primary tg-step", "tg-retry"));
  }
}
