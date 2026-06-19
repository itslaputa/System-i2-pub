import { Chart } from "chart.js/auto";
import { Show, createEffect, onCleanup } from "solid-js";
import type { AnalyticsRootShareItem } from "../../services/tauri/analytics";
import { formatHoursMinutes } from "./analyticsFormatting";
import { getRootCategoryColor } from "./analyticsPalette";

type AnalyticsPieChartProps = {
  items: AnalyticsRootShareItem[];
};

function formatPercent(value: number, total: number) {
  if (total <= 0) {
    return "0%";
  }

  return `${(value / total * 100).toFixed(1)}%`;
}

function AnalyticsPieChart(props: AnalyticsPieChartProps) {
  let canvasRef: HTMLCanvasElement | undefined;
  let chart: Chart | undefined;

  createEffect(() => {
    const items = props.items;
    const totalMinutes = items.reduce((sum, item) => sum + item.totalMinutes, 0);

    if (!canvasRef || items.length === 0) {
      chart?.destroy();
      chart = undefined;
      return;
    }

    chart?.destroy();
    chart = new Chart(canvasRef, {
      type: "pie",
      data: {
        labels: items.map(
          (item) => `${item.label} · ${formatPercent(item.totalMinutes, totalMinutes)}`,
        ),
        datasets: [
          {
            data: items.map((item) => item.totalMinutes),
            backgroundColor: items.map(
              (item) => getRootCategoryColor(item.label).backgroundColor,
            ),
            borderColor: items.map(
              (item) => getRootCategoryColor(item.label).borderColor,
            ),
            borderWidth: 2,
          },
        ],
      },
      options: {
        maintainAspectRatio: false,
        plugins: {
          legend: {
            position: "bottom",
            labels: {
              color: "#d9e5e2",
              boxWidth: 12,
              boxHeight: 12,
              padding: 18,
            },
          },
          tooltip: {
            callbacks: {
              label(context) {
                const value = Number(context.parsed ?? 0);
                return `${items[context.dataIndex]?.label ?? context.label}: ${formatHoursMinutes(value)} · ${formatPercent(value, totalMinutes)}`;
              },
            },
          },
        },
      },
    });
  });

  onCleanup(() => chart?.destroy());

  return (
    <div class="analytics-chart-frame">
      <Show
        when={props.items.length > 0}
        fallback={<p class="analytics-empty-state">Нет данных для круговой диаграммы.</p>}
      >
        <canvas ref={canvasRef} />
      </Show>
    </div>
  );
}

export default AnalyticsPieChart;
