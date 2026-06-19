import { describe, expect, it } from "vitest";
import {
  formatHoursMinutes,
  formatProjectPeriodLabel,
  formatRangeDate,
} from "../../../src/features/analytics/analyticsFormatting";

describe("analyticsFormatting", () => {
  it("formats range dates into readable russian labels", () => {
    expect(formatRangeDate("2026-03-13")).toBe("13 мар 2026");
    expect(formatRangeDate("oops")).toBe("oops");
  });

  it("shows sub-hour analytics durations as minutes only", () => {
    expect(formatHoursMinutes(0)).toBe("0мин");
    expect(formatHoursMinutes(55)).toBe("55мин");
    expect(formatHoursMinutes(125)).toBe("2ч 05мин");
  });

  it("maps project period state to readable labels", () => {
    expect(formatProjectPeriodLabel(true)).toBe("Закончен за период");
    expect(formatProjectPeriodLabel(false)).toBe("Был в работе");
  });
});
