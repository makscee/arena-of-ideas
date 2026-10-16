/// <reference types="vite/client" />
// The CrazyGames build (mission #1214, `npm run mvp:build:cg`): the same
// client, portable, with everything the portal forbids or doesn't need left
// out: off-portal links, the email, Telegram and invite logins, the dev tools.
export const CG: boolean = import.meta.env.VITE_CG === "1";

/** The API's origin when the client is not served by the API (VITE_API_ORIGIN,
 * no trailing slash); empty: the page's own base. */
export const API_ORIGIN: string = ((import.meta.env.VITE_API_ORIGIN as string | undefined) ?? "").replace(/\/+$/, "");

/** The API prefix's base: `${API_ORIGIN}` + MVP_API_PREFIX, or relative to the app's base. */
export const apiBase = (prefix: string): string => (API_ORIGIN || (import.meta.env.BASE_URL ?? "/").replace(/\/$/, "")) + prefix;
