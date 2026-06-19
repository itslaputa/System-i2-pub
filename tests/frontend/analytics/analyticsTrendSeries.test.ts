import { describe, expect, it } from "vitest";
import {
  buildTrendLineDataset,
  formatTrendAxisLabel,
  formatTrendModeLabel,
} from "../../../src/features/analytics/analyticsTrendSeries";

describe("analyticsTrendSeries", () => {
  it("builds total series with highlighted current point", () => {
    const dataset = buildTrendLineDataset({
      seriesId: "total",
      label: "Суммарное время",
      points: [
        {
          label: "янв 2026",
          startDate: "2026-01-01",
          endDate: "2026-01-31",
          totalMinutes: 120,
          isCurrent: false,
        },
        {
          label: "фев 2026",
          startDate: "2026-02-01",
          endDate: "2026-02-28",
          totalMinutes: 180,
          isCurrent: true,
        },
      ],
    });

    expect(dataset.data).toEqual([2, 3]);
    expect(dataset.pointRadius).toEqual([3, 5]);
    expect(dataset.pointBackgroundColor[1]).toBe("#ecf4f1");
  });

  it("maps trend modes to readable labels", () => {
    expect(formatTrendModeLabel("week")).toBe("По неделям");
    expect(formatTrendModeLabel("month")).toBe("По месяцам");
    expect(formatTrendModeLabel("year")).toBe("По годам");
    expect(formatTrendModeLabel("custom")).toBe("По периодам");
  });

  it("formats ranged trend points into compact single-line axis labels", () => {
    expect(
      formatTrendAxisLabel({
        label: "02 фев-08 фев",
        startDate: "2026-02-02",
        endDate: "2026-02-08",
        totalMinutes: 120,
        isCurrent: false,
      }),
    ).toBe("02-08 фев");
  });

  it("formats cross-month ranged points without duplicating the full year", () => {
    expect(
      formatTrendAxisLabel({
        label: "23 фев-01 мар",
        startDate: "2026-02-23",
        endDate: "2026-03-01",
        totalMinutes: 120,
        isCurrent: false,
      }),
    ).toBe("23 фев-01 мар");
  });

  it("keeps month and year trend labels as single-line labels", () => {
    expect(
      formatTrendAxisLabel({
        label: "мар 2026",
        startDate: "2026-03-01",
        endDate: "2026-03-31",
        totalMinutes: 300,
        isCurrent: true,
      }),
    ).toBe("мар 2026");
  });
});
