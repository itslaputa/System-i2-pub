import { describe, expect, it } from "vitest";
import { ANALYTICS_PRESETS, getPresetRange } from "../../../src/features/analytics/analyticsPresets";

describe("analyticsPresets", () => {
  it("includes a today preset", () => {
    expect(ANALYTICS_PRESETS.some((preset) => preset.key === "today")).toBe(true);
    expect(ANALYTICS_PRESETS.find((preset) => preset.key === "today")?.label).toBe(
      "Сегодня",
    );
  });

  it("maps the today preset to a single-day range", () => {
    const range = getPresetRange("today", new Date(2026, 3, 24));

    expect(range).toEqual({
      startDate: "2026-04-24",
      endDate: "2026-04-24",
    });
  });
});
