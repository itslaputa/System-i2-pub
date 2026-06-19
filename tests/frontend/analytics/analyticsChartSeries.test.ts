import { describe, expect, it } from "vitest";
import { buildStackedMinutesDatasets } from "../../../src/features/analytics/analyticsChartSeries";

describe("buildStackedMinutesDatasets", () => {
  it("keeps backend label order and fills missing days with zeros", () => {
    const datasets = buildStackedMinutesDatasets([
      {
        date: "2026-03-12",
        totalMinutes: 90,
        taskCount: 2,
        categoryBreakdown: [
          { categoryId: "money", label: "Деньги", totalMinutes: 30 },
          { categoryId: "logic", label: "Логика", totalMinutes: 60 },
        ],
      },
      {
        date: "2026-03-13",
        totalMinutes: 20,
        taskCount: 1,
        categoryBreakdown: [{ categoryId: "logic", label: "Логика", totalMinutes: 20 }],
      },
    ]);

    expect(datasets).toHaveLength(2);
    expect(datasets[0]?.label).toBe("Деньги");
    expect(datasets[0]?.data).toEqual([0.5, 0]);
    expect(datasets[1]?.label).toBe("Логика");
    expect(datasets[1]?.data).toEqual([1, 0.33]);
  });

  it("uses pastel fallback color for unknown root labels", () => {
    const [dataset] = buildStackedMinutesDatasets([
      {
        date: "2026-03-12",
        totalMinutes: 15,
        taskCount: 1,
        categoryBreakdown: [{ categoryId: "unknown", label: "Другое", totalMinutes: 15 }],
      },
    ]);

    expect(dataset?.backgroundColor).toBe("rgba(173, 184, 199, 0.88)");
    expect(dataset?.borderColor).toBe("rgba(138, 150, 166, 0.96)");
  });

  it("normalizes punctuated root labels before resolving palette colors", () => {
    const [dataset] = buildStackedMinutesDatasets([
      {
        date: "2026-03-12",
        totalMinutes: 20,
        taskCount: 1,
        categoryBreakdown: [
          { categoryId: "health", label: "Здоровье.", totalMinutes: 20 },
        ],
      },
    ]);

    expect(dataset?.backgroundColor).toBe("rgba(138, 214, 158, 0.92)");
    expect(dataset?.borderColor).toBe("rgba(98, 180, 121, 0.98)");
  });

  it("keeps health separate from the synthetic other bucket", () => {
    const datasets = buildStackedMinutesDatasets([
      {
        date: "2026-03-12",
        totalMinutes: 45,
        taskCount: 2,
        categoryBreakdown: [
          { categoryId: "health", label: "Здоровье.", totalMinutes: 30 },
          { categoryId: "__unknown__", label: "Остальное", totalMinutes: 15 },
        ],
      },
    ]);

    expect(datasets.map((dataset) => dataset.label)).toEqual([
      "Здоровье.",
      "Остальное",
    ]);
    expect(datasets[0]?.backgroundColor).toBe("rgba(138, 214, 158, 0.92)");
    expect(datasets[1]?.backgroundColor).toBe("rgba(173, 184, 199, 0.88)");
  });
});
