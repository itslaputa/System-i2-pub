import { describe, expect, it } from "vitest";
import { formatHoursMinutes } from "../../../src/utils/formatDuration";

describe("formatDuration", () => {
  it("formats minute totals as compact hours/minutes labels", () => {
    expect(formatHoursMinutes(0)).toBe("0ч 00мин");
    expect(formatHoursMinutes(55)).toBe("0ч 55мин");
    expect(formatHoursMinutes(125)).toBe("2ч 05мин");
  });

  it("clamps invalid inputs to zero", () => {
    expect(formatHoursMinutes(-10)).toBe("0ч 00мин");
    expect(formatHoursMinutes(Number.NaN)).toBe("0ч 00мин");
    expect(formatHoursMinutes(Number.POSITIVE_INFINITY)).toBe("0ч 00мин");
  });
});
