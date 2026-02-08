import * as utilA from "./util_a";
import * as utilB from "./util_b";

export function run(): number {
  return utilA.helper() + utilB.helper();
}
