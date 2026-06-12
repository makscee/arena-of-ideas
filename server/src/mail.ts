/**
 * void-mail client. Interface + mock copied from void-auth
 * (src/clients/mail.ts); the HTTP implementation uses plain fetch against
 * void-mail's POST /v1/mail/send instead of openapi-fetch, to avoid carrying
 * the generated-types toolchain into the arena.
 *
 * Tests always use the mock — no network in the suite.
 */

export type MailSendInput = {
  to: string;
  subject: string;
  html: string;
  text: string;
};

export type MailSendResult = { ok: true } | { ok: false; error: string };

export interface MailClient {
  send(msg: MailSendInput): Promise<MailSendResult>;
}

export interface MockMailClient extends MailClient {
  sent: MailSendInput[];
  reset(): void;
}

/** Truncate an error string to ~500 chars (void-auth convention). */
function truncateError(s: string): string {
  return s.length <= 500 ? s : s.slice(0, 500);
}

export function createMockMailClient(): MockMailClient {
  const sent: MailSendInput[] = [];
  return {
    sent,
    async send(msg) {
      sent.push(msg);
      return { ok: true };
    },
    reset() {
      sent.length = 0;
    },
  };
}

export function createMailClient(opts: { baseUrl: string; token: string }): MailClient {
  const base = opts.baseUrl.replace(/\/+$/, "");
  return {
    async send(msg) {
      try {
        const res = await fetch(`${base}/v1/mail/send`, {
          method: "POST",
          headers: {
            authorization: `Bearer ${opts.token}`,
            "content-type": "application/json",
          },
          body: JSON.stringify(msg),
        });
        if (res.ok) return { ok: true };
        const body = await res.text().catch(() => "");
        return { ok: false, error: truncateError(body || `HTTP ${res.status}`) };
      } catch (err) {
        return { ok: false, error: truncateError(String(err)) };
      }
    },
  };
}
