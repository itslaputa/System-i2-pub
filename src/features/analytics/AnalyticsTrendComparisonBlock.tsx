import { For } from "solid-js";
import type { AnalyticsTrendComparison } from "../../services/tauri/analytics";
import AnalyticsTrendLineChart from "./AnalyticsTrendLineChart";
import AnalyticsTrendTotalChart from "./AnalyticsTrendTotalChart";

type AnalyticsTrendComparisonBlockProps = {
  trendComparison: AnalyticsTrendComparison;
};

function AnalyticsTrendComparisonBlock(props: AnalyticsTrendComparisonBlockProps) {
  return (
    <article class="panel analytics-block-panel">
      <div class="analytics-block-header">
        <div class="analytics-block-heading">
          <div>
            <h2>Сравнение с предыдущими периодами</h2>
          </div>
        </div>
      </div>

      <div class="analytics-trend-grid">
        <div class="analytics-trend-grid-main">
          <AnalyticsTrendTotalChart
            title="Суммарное время"
            series={props.trendComparison.total}
          />
        </div>

        <For each={props.trendComparison.categories}>
          {(series) => (
            <AnalyticsTrendLineChart title={series.label} series={series} />
          )}
        </For>
      </div>
    </article>
  );
}

export default AnalyticsTrendComparisonBlock;
