import type { AnalyticsTrendSeries } from "../../services/tauri/analytics";
import { formatTrendAxisLabel } from "./analyticsTrendSeries";

export const TOTAL_TREND_FRAME_HEIGHT_PX = 300;
export const TOTAL_TREND_TRACK_HEIGHT_PX = 210;

export type AnalyticsTrendTotalBar = {
  label: string;
  totalMinutes: number;
  heightPercent: number;
  isCurrent: boolean;
  startDate: string;
  endDate: string;
};

export function buildTrendTotalBars(
  series: AnalyticsTrendSeries,
): AnalyticsTrendTotalBar[] {
  const maxMinutes = Math.max(...series.points.map((point) => point.totalMinutes), 0);

  return series.points.map((point) => ({
    label: formatTrendAxisLabel(point),
    totalMinutes: point.totalMinutes,
    heightPercent:
      maxMinutes <= 0
        ? 10
        : Math.max(10, Math.round((point.totalMinutes / maxMinutes) * 100)),
    isCurrent: point.isCurrent,
    startDate: point.startDate,
    endDate: point.endDate,
  }));
}
