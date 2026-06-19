import { Chart } from "chart.js/auto";
import { Show, createEffect, onCleanup } from "solid-js";
import type { AnalyticsTrendSeries } from "../../services/tauri/analytics";
import { buildTrendLineDataset, formatTrendAxisLabel } from "./analyticsTrendSeries";
import { formatHoursMinutes, formatRangeDate } from "./analyticsFormatting";

type AnalyticsTrendLineChartProps = {
  title: string;
  series: AnalyticsTrendSeries;
};

function AnalyticsTrendLineChart(props: AnalyticsTrendLineChartProps) {
  let canvasRef: HTMLCanvasElement | undefined;
  let chart: Chart | undefined;

  createEffect(() => {
    if (!canvasRef || props.series.points.length === 0) {
      chart?.destroy();
      chart = undefined;
      return;
    }

    const dataset = buildTrendLineDataset(props.series);
    const hasZeroValues = dataset.data.some((value) => value === 0);

    chart?.destroy();
    chart = new Chart(canvasRef, {
      type: "line",
      data: {
        labels: props.series.points.map((point) => formatTrendAxisLabel(point)),
        datasets: [
          {
            ...dataset,
            borderWidth: 3,
            tension: 0.24,
            fill: true,
            clip: false,
            pointBorderColor: dataset.borderColor,
            pointBorderWidth: 2,
            pointHoverRadius: 7,
          },
        ],
      },
      options: {
        maintainAspectRatio: false,
        layout: {
          padding: {
            bottom: hasZeroValues ? 14 : 6,
          },
        },
        scales: {
          x: {
            ticks: {
              color: "#9eb8b2",
              maxRotation: 0,
              minRotation: 0,
              autoSkip: false,
              padding: 10,
              font: { size: 11 },
            },
            grid: { color: "rgba(236, 244, 241, 0.08)" },
          },
          y: {
            beginAtZero: true,
            grace: "12%",
            ticks: { color: "#9eb8b2" },
            grid: { color: "rgba(236, 244, 241, 0.08)" },
          },
        },
        plugins: {
          legend: { display: false },
          tooltip: {
            callbacks: {
              title(items) {
                return items[0]?.label ?? "";
              },
              label(context) {
                const point = props.series.points[context.dataIndex];
                return `${props.series.label}: ${formatHoursMinutes(point?.totalMinutes ?? 0)}`;
              },
              afterLabel(context) {
                const point = props.series.points[context.dataIndex];
                if (!point) {
                  return undefined;
                }
                return `${formatRangeDate(point.startDate)} - ${formatRangeDate(point.endDate)}`;
              },
            },
          },
        },
      },
    });
  });

  onCleanup(() => chart?.destroy());

  return (
    <div class="analytics-trend-card">
      <div class="analytics-mini-heading">
        <p>{props.title}</p>
      </div>
      <div class="analytics-chart-frame is-series is-trend">
        <Show
          when={props.series.points.length > 0}
          fallback={<p class="analytics-empty-state">Нет данных для графика.</p>}
        >
          <canvas ref={canvasRef} />
        </Show>
      </div>
    </div>
  );
}

export default AnalyticsTrendLineChart;
