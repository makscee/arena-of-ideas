// clients/core (#957): what any Arena client needs, DOM-free and Node-free (a
// test checks its imports): the fetch client, the game as text, and the
// reference bot. The protocol itself is protocol/ (openapi.json, schema.json,
// battle-events.md).
export * from "./api.js";
export * from "./bot.js";
export * from "./text.js";
