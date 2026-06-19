import { Chart } from "chart.js/auto";
import { Show, createEffect, onCleanup } from "solid-js";
import type {
  AnalyticsDailyChartKind,
  AnalyticsDailyPoint,
} from "../../services/tauri/analytics";
import { buildStackedMinutesDatasets } from "./analyticsChartSeries";
import { formatHoursMinutes } from "./analyticsFormatting";

type AnalyticsTimeSeriesChartProps = {
  title: string;
  points: AnalyticsDailyPoint[];
  chartKind: AnalyticsDailyChartKind;
  valueKey: "totalMinutes" | "taskCount";
};

function formatAxisLabel(dateValue: string) {
  return dateValue.slice(5);
}

function AnalyticsTimeSeriesChart(props: AnalyticsTimeSeriesChartProps) {
  let canvasRef: HTMLCanvasElement | undefined;
  let chart: Chart | undefined;

  createEffect(() => {
    const points = props.points;

    if (!canvasRef || points.length === 0) {
      chart?.destroy();
      chart = undefined;
      return;
    }

    const isMinutesChart = props.valueKey === "totalMinutes";
    const useStackedRootBreakdown = isMinutesChart && props.chartKind === "bar";
    const dataValues = points.map((point) =>
      isMinutesChart ? Number((point.totalMinutes / 60).toFixed(2)) : point.taskCount,
    );
    const datasets = useStackedRootBreakdown
      ? buildStackedMinutesDatasets(points)
      : [
          {
            label: props.title,
            data: dataValues,
            borderColor: "#6fe1d1",
            backgroundColor:
              props.chartKind === "bar"
                ? "rgba(15, 123, 108, 0.52)"
                : "rgba(15, 123, 108, 0.16)",
            pointBackgroundColor: "#ecf4f1",
            pointBorderColor: "#0f7b6c",
            pointRadius: props.chartKind === "line" ? 3 : 0,
            pointHoverRadius: 5,
            borderWidth: 3,
            tension: props.chartKind === "line" ? 0.28 : 0,
            fill: props.chartKind === "line",
          },
        ];

    chart?.destroy();
    chart = new Chart(canvasRef, {
      type: props.chartKind,
      data: {
        labels: points.map((point) => formatAxisLabel(point.date)),
        datasets,
      },
      options: {
        maintainAspectRatio: false,
        scales: {
          x: {
            stacked: useStackedRootBreakdown,
            ticks: {
              color: "#9eb8b2",
              maxRotation: 0,
              autoSkip: true,
            },
            grid: {
              color: "rgba(236, 244, 241, 0.08)",
            },
          },
          y: {
            stacked: useStackedRootBreakdown,
            beginAtZero: true,
            ticks: {
              color: "#9eb8b2",
            },
            grid: {
              color: "rgba(236, 244, 241, 0.08)",
            },
          },
        },
        plugins: {
          legend: {
            display: useStackedRootBreakdown,
            labels: {
              color: "#b8cec8",
              usePointStyle: true,
              boxWidth: 10,
              boxHeight: 10,
              padding: 16,
            },
          },
          tooltip: {
            callbacks: {
              label(context) {
                if (isMinutesChart) {
                  if (useStackedRootBreakdown) {
                    const categorySlice = props.points[context.dataIndex]?.categoryBreakdown.find(
                      (categoryEntry) => categoryEntry.label === context.dataset.label,
                    );
                    const categoryMinutes = categorySlice?.totalMinutes ?? 0;

                    return `${context.dataset.label}: ${formatHoursMinutes(categoryMinutes)}`;
                  }

                  const originalMinutes =
                    props.points[context.dataIndex]?.totalMinutes ?? 0;
                  return `Часы: ${formatHoursMinutes(originalMinutes)}`;
                }

                return `Таски: ${context.parsed.y}`;
              },
              footer(tooltipItems) {
                if (!useStackedRootBreakdown || tooltipItems.length === 0) {
                  return undefined;
                }

                const originalMinutes =
                  props.points[tooltipItems[0]?.dataIndex ?? 0]?.totalMinutes ?? 0;
                return `Итого за день: ${formatHoursMinutes(originalMinutes)}`;
              },
            },
          },
        },
      },
    });
  });

  onCleanup(() => chart?.destroy());

  return (
    <div class="analytics-time-chart-card">
      <div class="analytics-mini-heading">
        <p>{props.title}</p>
      </div>
      <div class="analytics-chart-frame is-series">
        <Show
          when={props.points.length > 0}
          fallback={<p class="analytics-empty-state">Нет данных для графика.</p>}
        >
          <canvas ref={canvasRef} />
        </Show>
      </div>
    </div>
  );
}

export default AnalyticsTimeSeriesChart;
