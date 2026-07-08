import { readFileSync } from "node:fs";

const UNKNOWN = "unknown";
const SOURCE = "arena-of-ideas";

export interface BuildIdentity {
  source: typeof SOURCE;
  version: string;
  commit: string;
  image: string;
  buildTime: string;
}

function envValue(env: NodeJS.ProcessEnv, names: string[]): string {
  for (const name of names) {
    const value = env[name]?.trim();
    if (value) return value;
  }
  return UNKNOWN;
}

function packageVersion(): string {
  try {
    const body = JSON.parse(readFileSync(new URL("../../package.json", import.meta.url), "utf8")) as {
      version?: unknown;
    };
    return typeof body.version === "string" && body.version.trim() !== "" ? body.version : UNKNOWN;
  } catch {
    return UNKNOWN;
  }
}

export function readBuildIdentity(env: NodeJS.ProcessEnv = process.env): BuildIdentity {
  const version = envValue(env, ["ARENA_APP_VERSION"]);
  return {
    source: SOURCE,
    version: version === UNKNOWN ? packageVersion() : version,
    commit: envValue(env, ["ARENA_BUILD_COMMIT", "GIT_COMMIT", "SOURCE_COMMIT", "COMMIT_SHA"]),
    image: envValue(env, ["ARENA_BUILD_IMAGE", "IMAGE_TAG"]),
    buildTime: envValue(env, ["ARENA_BUILD_TIME", "BUILD_DATE"]),
  };
}
