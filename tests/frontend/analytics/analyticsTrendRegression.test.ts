import analyticsTrendComparisonBlockSource from "../../../src/features/analytics/AnalyticsTrendComparisonBlock.tsx?raw";
import analyticsTrendTotalChartSource from "../../../src/features/analytics/AnalyticsTrendTotalChart.tsx?raw";
import { describe, expect, it } from "vitest";
import {
  TOTAL_TREND_FRAME_HEIGHT_PX,
  TOTAL_TREND_TRACK_HEIGHT_PX,
} from "../../../src/features/analytics/analyticsTrendTotalBars";

describe("analytics trend regressions", () => {
  it("keeps the total trend isolated from the mini line charts", () => {
    expect(analyticsTrendComparisonBlockSource).toContain(
      'import AnalyticsTrendTotalChart',
    );
    expect(analyticsTrendComparisonBlockSource).toContain(
      "<AnalyticsTrendTotalChart",
    );
    expect(analyticsTrendComparisonBlockSource).not.toContain('variant="total"');
  });

  it("uses explicit heights for the total trend container and track", () => {
    expect(TOTAL_TREND_FRAME_HEIGHT_PX).toBe(300);
    expect(TOTAL_TREND_TRACK_HEIGHT_PX).toBe(210);
    expect(TOTAL_TREND_TRACK_HEIGHT_PX).toBeLessThan(TOTAL_TREND_FRAME_HEIGHT_PX);
    expect(analyticsTrendTotalChartSource).toContain("TOTAL_TREND_FRAME_HEIGHT_PX");
    expect(analyticsTrendTotalChartSource).toContain("TOTAL_TREND_TRACK_HEIGHT_PX");
  });
});
