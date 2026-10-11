// The HTTP API's route table (#957): every path under MVP_API_PREFIX, its body
// and its answer, named by their types in ./contract.ts. scripts/protocol.ts
// builds protocol/openapi.json from it, and a test checks that it lists exactly
// the routes server/src/mvp/app.ts serves.
//
// The promise (protocol/README.md): /api/v1 only gains things (fields,
// routes, types). Renaming or removing one means /api/v2.

/** A type exported from ./contract.ts (or ../types.ts), by name; `X[]` is a list of them. */
export type ProtocolTypeName = string;

export interface ApiRoute {
  method: "GET" | "POST";
  /** Under MVP_API_PREFIX, Hono style: `:runId` is a path parameter. */
  path: string;
  /** The JSON body's type; none for a call without one. */
  body?: ProtocolTypeName;
  /** The body may be left out. */
  bodyOptional?: true;
  /** The 200 answer's type, or a media type for a file. */
  response: ProtocolTypeName | "image/png" | "application/json";
  /** Off unless the server runs with it: `dev` is MVP_DEV=1 (and an admin on
   * an invite-only server), the fakes are ARENA_TELEGRAM_FAKE / ARENA_MAIL_FAKE. */
  only?: "dev" | "fake-telegram" | "fake-mail";
  /** #958, assist only: may an agent client (CLIENT_HEADER) call this POST?
   * Every GET is open to it; a POST without `agent` answers it 403. "practice":
   * only on a practice run (POST /runs: with mode "practice"); "logged-out":
   * only without a session (with one it would link a login to the account). */
  agent?: true | "practice" | "logged-out";
  /** What it does and its error statuses beyond the common ones. */
  note: string;
}

export const API_ROUTES: readonly ApiRoute[] = [
  { method: "GET", path: "/health", response: "HealthView", note: "The server: API version, content version, deployed build, and which logins are on." },
  { method: "GET", path: "/content", response: "MvpContent", note: "The live pool: rules, units, summons, glossary." },
  { method: "POST", path: "/players", body: "NameRequest", response: "PlayerRef", agent: true, note: "A device identity on an open server. 403 on an invite-only server." },
  { method: "POST", path: "/invites/lookup", body: "CodeRequest", response: "InviteLookup", agent: true, note: "Whose invite link it is, opening nothing (slice 13). 404 unknown code." },
  { method: "POST", path: "/invites/redeem", body: "CodeRequest", response: "PlayerSession", agent: true, note: "Opens an invite link: the same link again is the same player with a new token; a player keeps their newest 10 devices' tokens. 404 unknown code." },
  { method: "POST", path: "/join/check", body: "CodeRequest", response: "OkResponse", agent: true, note: "Is this the open join link (R4-20)? 404 if not." },
  { method: "POST", path: "/join", body: "JoinRequest", response: "JoinSession", agent: true, note: "A new player through the open join link, never admin, with their own invite. 404 not the join code, 409 a taken or bot's name, 400 a bad name, 429 over 30 joins an hour." },
  { method: "POST", path: "/auth/email/start", body: "EmailStartRequest", response: "EmailStartResponse", agent: true, note: "Mails a 6-digit code to `email`, for sign-in or sign-up, or with a session to attach it; never says whether the email has a player (#929). 400 a bad email or name, 404 email login off, 429 a code a minute per email or too many this hour, 502 not sent." },
  { method: "POST", path: "/auth/email/verify", body: "EmailVerifyRequest", response: "EmailVerify", agent: true, note: "A known email logs in; a new one with `name` makes the player (with `join` on an invite-only server); without `name` \"needsName\" and the code stays. 400 a wrong code (5 tries), 404 no code or expired, 403 invite only, 409 name taken." },
  { method: "POST", path: "/auth/email/attach", body: "EmailAttachRequest", response: "EmailStatus", note: "Ties the email to the caller, who has none yet, with the code from a start sent with their session. 409 the email has a player." },
  { method: "GET", path: "/auth/email", response: "EmailStatus", note: "Is email login on, and the caller's email." },
  { method: "POST", path: "/auth/telegram/start", response: "TelegramStart", agent: "logged-out", note: "A one-time login code and its t.me deep link, for the caller (Link Telegram) or for none (Log in) (M4-6). 404 Telegram login off, 403 a bot or an admin, 409 already linked, 429 over 20 an hour." },
  { method: "POST", path: "/auth/telegram/poll", body: "CodeRequest", response: "TelegramPoll", agent: true, note: "\"waiting\" until Yes in the bot, then the session, once; \"declined\" after No. 404 unknown, expired or used: start again." },
  { method: "GET", path: "/auth/telegram", response: "TelegramStatus", note: "Is Telegram login on, is the caller linked." },
  { method: "POST", path: "/auth/telegram/unlink", response: "TelegramStatus", note: "Unlinks the caller's Telegram." },
  { method: "GET", path: "/home", response: "HomeView", note: "The home screen: rules, the day, the caller's rating, active run and practice run." },
  { method: "POST", path: "/runs", body: "StartRunRequest", bodyOptional: true, response: "RunView", agent: "practice", note: "Starts a run, ranked unless `mode` is \"practice\" (#958); the player's run of that mode if one is going." },
  { method: "GET", path: "/runs/:runId", response: "RunView", note: "A run as its player sees it." },
  { method: "POST", path: "/runs/:runId/decisions", body: "Decision", response: "DecisionResponse", agent: "practice", note: "One move. 400 a Decision of an unknown kind, 409 a move the rules refuse." },
  { method: "POST", path: "/runs/:runId/preview", body: "Decision", response: "DecisionResponse", agent: true, note: "A dry run of one move, no writes. 400 for a fight." },
  { method: "POST", path: "/runs/:runId/abandon", response: "RunView", agent: "practice", note: "Gives the run up: endedBy \"abandoned\". 409 when over." },
  { method: "GET", path: "/battles/:battleId", response: "BattleRecord", note: "A stored battle with its full log (battle-events/1)." },
  { method: "GET", path: "/ideas", response: "MyIdeasView", note: "The caller's own ideas only (M2-4). 401 without a player." },
  { method: "POST", path: "/ideas", body: "WriteIdeaRequest", response: "MyIdeasView", note: "Writes one, spending a held idea (M2-4). 400 text not IDEA_TEXT_MIN–MAX characters, 409 no idea held; kind \"evolve\" proposes a new version of `target` (M3-4): 404 no such unit, 409 the target isn't in the Library." },
  { method: "POST", path: "/ideas/:ideaId/cancel", response: "MyIdeasView", note: "Takes back a `written` idea and refunds it. 404 not the caller's, 409 past `written` or ideas held at the cap." },
  { method: "GET", path: "/ideas/:ideaId", response: "MyIdea", note: "The caller's own idea with its options (M2-5). 404 not theirs." },
  { method: "POST", path: "/ideas/:ideaId/archetype", body: "IndexRequest", response: "MyIdeasView", note: "Picks an archetype in `pick-archetype`: the reader reads its readings next. 400 no such option, 409 another state." },
  { method: "POST", path: "/ideas/:ideaId/reading", body: "IndexRequest", response: "MyIdeasView", note: "Picks a reading in `pick-reading`: the idea is `simulating`, its unit a candidate. 409 another state, or the name or shape was just taken, the option dropped." },
  { method: "POST", path: "/ideas/:ideaId/none", response: "MyIdeasView", note: "\"None of these\" in `pick-archetype` or `pick-reading`: the stage is read once more; the second time the idea fails and is refunded (M2-6). 409 another state." },
  { method: "GET", path: "/votes/next", response: "VoteCardResponse", note: "Never the caller's own candidate, nor a pair they voted on (M2-8); new versions, and \"unchanged\" open to anyone (M3-6)." },
  { method: "POST", path: "/votes", body: "VoteRequest", response: "VoteCardResponse", note: "Votes, and the next card. 404 no such candidate or live unit, 400 a pick of neither, 409 already voted on this pair, or the caller's own candidate, or a bot." },
  { method: "GET", path: "/fusions", response: "FusionDiscovery[]", note: "Every fusion found so far, with its credit (slice 10)." },
  { method: "GET", path: "/day", response: "DayView", note: "Today: the champion, slayers, the throne (slice 5)." },
  { method: "GET", path: "/stats", response: "StatsView", note: "Unit win and pick rates, champions (slice 11)." },
  { method: "GET", path: "/credits", response: "CreditsView", note: "Authors, NEW, the caller's creator number (M2-9)." },
  { method: "GET", path: "/library", response: "LibraryView", note: "The units that have left the pool (M2-9)." },
  { method: "GET", path: "/share/champion/:file", response: "image/png", note: "`<day>.png`: the champion's share card (M4-7); ?lang=en|ru." },
  { method: "GET", path: "/share/unit/:file", response: "image/png", note: "`<unitId>.png`: a unit's share card (M4-7); ?lang=en|ru." },
  { method: "GET", path: "/schema.json", response: "application/json", note: "This protocol's types (protocol/schema.json, #957)." },
  { method: "GET", path: "/openapi.json", response: "application/json", note: "This route table as OpenAPI 3.1 (protocol/openapi.json, #957)." },
  { method: "POST", path: "/dev/telegram/accept", body: "TelegramAcceptRequest", response: "ReplyResponse", only: "fake-telegram", note: "The fake bot's user sends /start <code>, then taps Yes, or No with decline: true." },
  { method: "POST", path: "/dev/mail/last", body: "MailLastRequest", response: "MailLast", only: "fake-mail", agent: true, note: "The last code mailed to `email`." },
  { method: "POST", path: "/dev/end-day", response: "DayView", only: "dev", note: "Ends the day now (slice 5)." },
  { method: "POST", path: "/dev/grant-idea", response: "IdeasView", only: "dev", note: "+1 idea up to the cap (M2-3)." },
  { method: "POST", path: "/dev/credit-unit", body: "UnitIdRequest", response: "CreditsView", only: "dev", note: "The unit becomes the caller's idea, entered today (M2-9)." },
  { method: "POST", path: "/dev/overnight-check", response: "StartedResponse", only: "dev", note: "Starts the simulation on every `simulating` idea (M2-8)." },
  { method: "POST", path: "/dev/seed-candidate", body: "SeedCandidateRequest", response: "MyIdeasView", only: "dev", note: "The caller's idea in `simulating`: a renamed live unit, `unit`'s or a random one (M2-8)." },
  { method: "POST", path: "/dev/fake-votes", response: "CandidateScore[]", only: "dev", note: "5 fake votes on each candidate, 4 for it (M2-8)." },
  { method: "GET", path: "/dev/candidates", response: "CandidateScore[]", only: "dev", note: "Every candidate, qualified ones first, best first (M2-8)." },
];
