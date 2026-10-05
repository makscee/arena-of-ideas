import { describeMvpStore } from "./store.contract.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { MemoryMvpStore } from "./store.js";

describeMvpStore("memory", () => new MemoryMvpStore());
describeMvpStore("sqlite", () => new SqliteMvpStore(":memory:"));
