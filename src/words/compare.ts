// The comparison the count conditions share (§3.2).

import type { CompareOp } from "../types.js";

export const compare = (v: number, op: CompareOp, n: number): boolean => (op === "<=" ? v <= n : op === ">=" ? v >= n : v === n);
export const opEn = (op: CompareOp): string => (op === "<=" ? "at most" : op === ">=" ? "at least" : "exactly");
export const opRu = (op: CompareOp): string => (op === "<=" ? "не больше" : op === ">=" ? "не меньше" : "ровно");
