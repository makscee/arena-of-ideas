import { defineConfig } from "vitest/config";

// vite.config.ts sets root: "web" for the web shell; vitest would inherit that
// and find no tests. This file takes priority for vitest and keeps the repo
// root, so the kernel suite under src/ runs as before.
//
// Half the cores, not all but one (R4-19): the suite runs beside other
// workers' suites and e2e on a shared Mac, and a worker starved past 60 s
// fails the run with "Timeout calling onTaskUpdate". VITEST_MAX_WORKERS
// overrides it on an idle machine.
export default defineConfig({
  test: { maxWorkers: process.env.VITEST_MAX_WORKERS ?? "50%" },
});
