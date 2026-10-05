import { describeMvpStore } from "./store.contract.js";
import { MemoryMvpStore } from "./store.js";

// Slice 4 adds: describeMvpStore("sqlite", () => new SqliteMvpStore(":memory:")).
describeMvpStore("memory", () => new MemoryMvpStore());
