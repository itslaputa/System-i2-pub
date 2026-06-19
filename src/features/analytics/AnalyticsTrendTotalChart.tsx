import { For } from "solid-js";
import type { AnalyticsTrendSeries } from "../../services/tauri/analytics";
import { formatHoursMinutes, formatRangeDate } from "./analyticsFormatting";
import {
  buildTrendTotalBars,
  TOTAL_TREND_FRAME_HEIGHT_PX,
  TOTAL_TREND_TRACK_HEIGHT_PX,
} from "./analyticsTrendTotalBars";

type AnalyticsTrendTotalChartProps = {
  title: string;
  series: AnalyticsTrendSeries;
};

function AnalyticsTrendTotalChart(props: AnalyticsTrendTotalChartProps) {
  const bars = () => buildTrendTotalBars(props.series);

  return (
    <div class="analytics-trend-card">
      <div class="analytics-mini-heading">
        <p>{props.title}</p>
      </div>
      <div
        class="analytics-chart-frame is-series is-trend is-total-trend"
        style={{ height: `${TOTAL_TREND_FRAME_HEIGHT_PX}px` }}
      >
        <div class="analytics-total-trend">
          <div class="analytics-total-trend-bars">
            <For each={bars()}>
              {(bar) => (
                <div class="analytics-total-trend-column">
                  <p class="analytics-total-trend-value">
                    {formatHoursMinutes(bar.totalMinutes)}
                  </p>
                  <div
                    class="analytics-total-trend-track"
                    style={{ height: `${TOTAL_TREND_TRACK_HEIGHT_PX}px` }}
                  >
                    <div
                      class="analytics-total-trend-bar"
                      classList={{ "is-current": bar.isCurrent }}
                      style={{ height: `${bar.heightPercent}%` }}
                      title={`${formatRangeDate(bar.startDate)} - ${formatRangeDate(bar.endDate)}`}
                    />
                  </div>
                  <p class="analytics-total-trend-label">{bar.label}</p>
                </div>
              )}
            </For>
          </div>
        </div>
      </div>
    </div>
  );
}

export default AnalyticsTrendTotalChart;
