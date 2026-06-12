/**
 * OTP email template. Copied from void-auth (src/services/otp-email.ts),
 * rebranded for the arena.
 */

export interface RenderOtpEmailInput {
  code: string;
  /** Seconds. */
  ttl: number;
}

export interface RenderedEmail {
  subject: string;
  html: string;
  text: string;
}

export function renderOtpEmail({ code, ttl }: RenderOtpEmailInput): RenderedEmail {
  const subject = `Your Arena of Ideas code: ${code}`;
  const html = `<p>Your login code is: <strong>${code}</strong></p>\n<p>Expires in ${Math.floor(ttl / 60)} minutes.</p>\n<p>If you didn't request this, ignore this email.</p>`;
  const text = `Your Arena of Ideas login code: ${code}\n\nExpires in ${Math.floor(ttl / 60)} minutes.\n\nIf you didn't request this, ignore this email.`;
  return { subject, html, text };
}
