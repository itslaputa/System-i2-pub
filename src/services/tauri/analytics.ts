import { invoke } from "@tauri-apps/api/core";

export type AnalyticsDashboardInput = {
  startDate: string;
  endDate: string;
};

export type AnalyticsProjectPeriodItem = {
  projectId: string;
  label: string;
  totalMinutes: number;
  finishedInPeriod: boolean;
};

export type AnalyticsProjectSummaryItem = {
  projectId: string;
  label: string;
  totalMinutes: number;
  taskCount: number;
  startDate: string | null;
  endDate: string | null;
  isDone: boolean;
  finishedInPeriod: boolean;
  categoryBreakdown: AnalyticsProjectCategoryBreakdownItem[];
};

export type AnalyticsProjectCategoryBreakdownItem = {
  categoryId: string;
  label: string;
  totalMinutes: number;
  taskCount: number;
};

export type AnalyticsCategorySummaryNode = {
  categoryId: string;
  label: string;
  totalMinutes: number;
  taskCount: number;
  projects: AnalyticsProjectPeriodItem[];
  children: AnalyticsCategorySummaryNode[];
};

export type AnalyticsRootShareItem = {
  categoryId: string;
  label: string;
  totalMinutes: number;
};

export type AnalyticsDailyPoint = {
  date: string;
  totalMinutes: number;
  taskCount: number;
  categoryBreakdown: AnalyticsDailyCategorySlice[];
};

export type AnalyticsDailyCategorySlice = {
  categoryId: string;
  label: string;
  totalMinutes: number;
};

export type AnalyticsCommentedTaskItem = {
  taskId: string;
  date: string;
  categoryLabel: string;
  projectLabel: string | null;
  timeLength: number;
  note: string;
};

export type AnalyticsDailyChartKind = "bar" | "line";

export type AnalyticsTrendMode = "week" | "month" | "year" | "custom";

export type AnalyticsTrendPoint = {
  label: string;
  startDate: string;
  endDate: string;
  totalMinutes: number;
  isCurrent: boolean;
};

export type AnalyticsTrendSeries = {
  seriesId: string;
  label: string;
  points: AnalyticsTrendPoint[];
};

export type AnalyticsTrendComparison = {
  mode: AnalyticsTrendMode;
  total: AnalyticsTrendSeries;
  categories: AnalyticsTrendSeries[];
};

export type AnalyticsDashboardData = {
  startDate: string;
  endDate: string;
  totalMinutes: number;
  categorySummaries: AnalyticsCategorySummaryNode[];
  projectSummaries: AnalyticsProjectSummaryItem[];
  rootShareItems: AnalyticsRootShareItem[];
  dailySeries: AnalyticsDailyPoint[];
  commentedTasks: AnalyticsCommentedTaskItem[];
  dailyChartKind: AnalyticsDailyChartKind;
  trendComparison: AnalyticsTrendComparison;
};

export function loadAnalyticsDashboard(input: AnalyticsDashboardInput) {
  return invoke<AnalyticsDashboardData>("load_analytics_dashboard", {
    input,
  });
}
