# clients/core

What any Arena client needs, DOM-free and Node-free (only `fetch`): the phone app, `npm run
mvp:bot`, and next the MCP server and the Claude Code mod. `core.test.ts` checks its imports.

```ts
import { ArenaClient, apiBase, botDecision, runText, battleText, traceText, cardText, glossaryText } from "./index.js";

const arena = new ArenaClient({ api: apiBase("https://arena.makscee.ru"), token: process.env.ARENA_TOKEN });
const content = await arena.content();
const run = await arena.startRun();
console.log(runText(run, content, (await arena.home()).rules));     // shop, line, bench, numbered as Decisions take them
const { run: next, fight } = await arena.decide(run.runId, botDecision(run, content, (await arena.home()).rules, 0));
if (fight) console.log(battleText(await arena.battle(fight.battleId), { you: "A" }));  // rows end with #<event id>
```

- `api.ts`: `apiCall` and `ArenaClient` (the routes are `protocol/openapi.json`).
- `text.ts`: cards, the run, battles turn by turn, the why-chain of an event, the glossary; `lang: "ru"` for Russian.
- `bot.ts`: the reference bot's next move, the one the server's bots play.
