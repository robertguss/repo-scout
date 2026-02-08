import * as utilA from "./util_a";
import * as utilB from "./util_b";
import { helper as helperA } from "./util_a";
import { helper as helperB } from "./util_b";

export function run_namespace_a(): number {
  return utilA.helper();
}

export function run_namespace_b(): number {
  return utilB.helper();
}

export function run_alias_a(): number {
  return helperA();
}

export function run_alias_b(): number {
  return helperB();
}
