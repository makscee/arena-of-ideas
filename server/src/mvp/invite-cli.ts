/**
 * Invite links for the Arena MVP (mission #574, slice 13), made on the host
 * against the server's SQLite file (safe while the server runs).
 *
 *   npm run mvp:invite -- list
 *   npm run mvp:invite -- add <name> [--claim | --player <id>] [--admin | --no-admin]
 *   npm run mvp:invite -- revoke <name>
 *   npm run mvp:invite -- open [--rotate]
 *
 * `add` prints the person's link. A new name makes a new player. A name an
 * existing human player has is refused unless --claim (the only human player
 * with that name) or --player <id> claims that player (the ids are listed), so
 * they keep their runs, rating and fusions. --admin lets
 * that player use the dev tools ("End day now"). A player who already has a
 * link gets the same link back; --admin / --no-admin on it promotes or demotes
 * them. Links last until revoked: `revoke` ends every device's session and
 * prints the person's new link (the old one stops working). Env: MVP_DB (default data/arena-mvp.db),
 * MVP_PUBLIC_URL (default https://arena.makscee.ru/arena/, where testers play).
 * `open` (R4-20) prints the one shared join link (…#join=<code>): anyone
 * who opens it picks a name and plays as a new player. The same link comes
 * back until `open --rotate` makes a new one; the old link then stops making
 * players, and those who joined keep their sessions and own links.
 * The code rides in the link's fragment (#invite=…, #join=…), which browsers
 * never send, so no proxy's access log holds it.
 */
import { SqliteMvpStore } from "./sqlite-store.js";
import { createInvite, InviteError, openJoin, revokeInvite } from "./invites.js";

const [cmd, ...rest] = process.argv.slice(2);
const store = new SqliteMvpStore(process.env.MVP_DB ?? "data/arena-mvp.db");
const base = process.env.MVP_PUBLIC_URL ?? "https://arena.makscee.ru/arena/";
const link = (code: string) => `${base}#invite=${code}`;

function flag(name: string): string | undefined {
  const i = rest.indexOf(name);
  if (i < 0) return undefined;
  const v = rest[i + 1];
  rest.splice(i, 2);
  return v;
}

try {
  if (cmd === "list") {
    for (const i of store.invites()) console.log(`${i.name}\t${i.playerId}\t${i.admin ? "admin" : "-"}\t${i.redeemedAt ? `opened ${i.redeemedAt}` : "not opened"}\t${link(i.code)}`);
  } else if (cmd === "add") {
    const promote = rest.includes("--admin");
    if (promote) rest.splice(rest.indexOf("--admin"), 1);
    const demote = rest.includes("--no-admin");
    if (demote) rest.splice(rest.indexOf("--no-admin"), 1);
    if (promote && demote) throw new InviteError("--admin or --no-admin, not both");
    const admin = promote ? true : demote ? false : undefined;
    const claim = rest.includes("--claim");
    if (claim) rest.splice(rest.indexOf("--claim"), 1);
    let playerId = flag("--player");
    const name = rest.join(" ");
    if (claim && !playerId) {
      const named = store.playersNamed(name);
      if (named.length !== 1) throw new InviteError(`--claim needs exactly one human player named ${name}, found ${named.length}: pick one with --player <id>`);
      playerId = named[0]!.id;
    }
    {
      const i = createInvite(store, { name, playerId, admin, now: new Date() });
      console.log(`${i.name}${i.admin ? " (admin)" : ""}: ${link(i.code)}`);
    }
  } else if (cmd === "revoke") {
    const name = rest.join(" ");
    const r = revokeInvite(store, name, new Date());
    if (!r) throw new InviteError(`no invite named ${name}`);
    console.log(`${r.invite.name}: the old link is dead, ${r.sessions} device session(s) ended. New link: ${link(r.invite.code)}`);
  } else if (cmd === "open" && (rest.length === 0 || (rest.length === 1 && rest[0] === "--rotate"))) {
    const rotate = rest[0] === "--rotate";
    rest.length = 0;
    const code = openJoin(store, rotate);
    console.log(`${rotate ? "New join link (the old one makes no new players)" : "Join link"}: ${base}#join=${code}`);
  } else {
    console.error("usage: mvp:invite -- list | add <name> [--claim | --player <id>] [--admin | --no-admin] | revoke <name> | open [--rotate]");
    process.exitCode = 2;
  }
} catch (e) {
  if (!(e instanceof InviteError)) throw e;
  console.error(e.message);
  const name = rest.join(" ");
  for (const p of store.playersNamed(name)) console.error(`  ${p.id}  ${p.name}  rating ${store.rating(p.id)?.rating ?? "-"}  runs ${store.rating(p.id)?.runs ?? 0}`);
  process.exitCode = 1;
} finally {
  store.close();
}
