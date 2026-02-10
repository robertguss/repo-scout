import { computePlan } from "./service";

describe("computePlan", () => {
  it("returns value", () => {
    expect(computePlan()).toBe(1);
  });
});
