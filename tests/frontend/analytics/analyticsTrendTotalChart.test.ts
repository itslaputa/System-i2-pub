import { describe, expect, it } from "vitest";
import { buildTrendTotalBars } from "../../../src/features/analytics/analyticsTrendTotalBars";

describe("analyticsTrendTotalChart", () => {
  it("builds proportional bars and highlights the current period", () => {
    const bars = buildTrendTotalBars({
      seriesId: "total",
      label: "Суммарное время",
      points: [
        {
          label: "02 фев-08 фев",
          startDate: "2026-02-02",
          endDate: "2026-02-08",
          totalMinutes: 120,
          isCurrent: false,
        },
        {
          label: "09 мар-13 мар",
          startDate: "2026-03-09",
          endDate: "2026-03-13",
          totalMinutes: 300,
          isCurrent: true,
        },
      ],
    });

    expect(bars).toHaveLength(2);
    expect(bars[0]?.label).toBe("02-08 фев");
    expect(bars[0]?.heightPercent).toBeLessThan(bars[1]?.heightPercent ?? 0);
    expect(bars[1]?.isCurrent).toBe(true);
  });

  it("keeps zero-only series visible with a minimum bar height", () => {
    const bars = buildTrendTotalBars({
      seriesId: "total",
      label: "Суммарное время",
      points: [
        {
          label: "02 фев-08 фев",
          startDate: "2026-02-02",
          endDate: "2026-02-08",
          totalMinutes: 0,
          isCurrent: false,
        },
      ],
    });

    expect(bars[0]?.heightPercent).toBe(10);
  });
});
