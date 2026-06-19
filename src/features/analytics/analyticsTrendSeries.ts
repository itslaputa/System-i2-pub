import type {
  AnalyticsTrendPoint,
  AnalyticsTrendSeries,
} from "../../services/tauri/analytics";
import { getRootCategoryColor } from "./analyticsPalette";

export type AnalyticsTrendLineDataset = {
  label: string;
  data: number[];
  borderColor: string;
  backgroundColor: string;
  pointBackgroundColor: string[];
  pointRadius: number[];
};

export function buildTrendLineDataset(
  series: AnalyticsTrendSeries,
): AnalyticsTrendLineDataset {
  const color =
    series.seriesId === "total"
      ? {
          backgroundColor: "rgba(111, 225, 209, 0.18)",
          borderColor: "#6fe1d1",
        }
      : getRootCategoryColor(series.label);

  return {
    label: series.label,
    data: series.points.map((point) =>
      Number((point.totalMinutes / 60).toFixed(2)),
    ),
    borderColor: color.borderColor,
    backgroundColor: color.backgroundColor,
    pointBackgroundColor: series.points.map((point) =>
      point.isCurrent ? "#ecf4f1" : color.borderColor,
    ),
    pointRadius: series.points.map((point) => (point.isCurrent ? 5 : 3)),
  };
}

export function formatTrendModeLabel(mode: string) {
  switch (mode) {
    case "week":
      return "По неделям";
    case "month":
      return "По месяцам";
    case "year":
      return "По годам";
    default:
      return "По периодам";
  }
}

export function formatTrendAxisLabel(
  point: AnalyticsTrendPoint,
): string {
  if (!point.label.includes("-")) {
    return point.label;
  }

  return formatCompactRange(point.startDate, point.endDate);
}

function formatCompactRange(startDate: string, endDate: string) {
  const [startYear, startMonth, startDay] = startDate.split("-");
  const [endYear, endMonth, endDay] = endDate.split("-");
  const shortMonths = [
    "янв",
    "фев",
    "мар",
    "апр",
    "май",
    "июн",
    "июл",
    "авг",
    "сен",
    "окт",
    "ноя",
    "дек",
  ];
  const startMonthIndex = Number(startMonth) - 1;
  const endMonthIndex = Number(endMonth) - 1;

  if (
    !startYear ||
    !startMonth ||
    !startDay ||
    !endYear ||
    !endMonth ||
    !endDay ||
    !shortMonths[startMonthIndex] ||
    !shortMonths[endMonthIndex]
  ) {
    return `${startDate}-${endDate}`;
  }

  if (startYear === endYear && startMonth === endMonth) {
    return `${startDay}-${endDay} ${shortMonths[startMonthIndex]}`;
  }

  return `${startDay} ${shortMonths[startMonthIndex]}-${endDay} ${shortMonths[endMonthIndex]}`;
}
