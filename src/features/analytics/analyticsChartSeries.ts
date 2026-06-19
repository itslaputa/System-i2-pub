import type { AnalyticsDailyPoint } from "../../services/tauri/analytics";
import { getRootCategoryColor } from "./analyticsPalette";

type StackedMinutesDataset = {
  label: string;
  data: number[];
  backgroundColor: string;
  borderColor: string;
  borderWidth: number;
  borderRadius: number;
  borderSkipped: false;
};

export function buildStackedMinutesDatasets(
  points: AnalyticsDailyPoint[],
): StackedMinutesDataset[] {
  const orderedLabels = Array.from(
    new Set(
      points.flatMap((point) =>
        point.categoryBreakdown.map((categorySlice) => categorySlice.label),
      ),
    ),
  );

  return orderedLabels.map((label) => {
    const colors = getRootCategoryColor(label);

    return {
      label,
      data: points.map((point) => {
        const matchedCategory = point.categoryBreakdown.find(
          (categorySlice) => categorySlice.label === label,
        );
        return Number((((matchedCategory?.totalMinutes ?? 0) as number) / 60).toFixed(2));
      }),
      backgroundColor: colors.backgroundColor,
      borderColor: colors.borderColor,
      borderWidth: 0,
      borderRadius: 0,
      borderSkipped: false,
    };
  });
}
