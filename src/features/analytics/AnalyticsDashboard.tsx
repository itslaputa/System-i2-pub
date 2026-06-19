import { For, Show, batch, createMemo, createResource, createSignal } from "solid-js";
import {
  loadAnalyticsDashboard,
  type AnalyticsDailyChartKind,
} from "../../services/tauri/analytics";
import AnalyticsCategoryTreeBlock, {
  type AnalyticsCategoryPathOrder,
} from "./AnalyticsCategoryTreeBlock";
import AnalyticsPieChart from "./AnalyticsPieChart";
import AnalyticsTrendComparisonBlock from "./AnalyticsTrendComparisonBlock";
import AnalyticsTimeSeriesChart from "./AnalyticsTimeSeriesChart";
import {
  ANALYTICS_PRESETS,
  getPresetRange,
  type AnalyticsPresetKey,
} from "./analyticsPresets";
import {
  formatHoursMinutes,
  formatProjectPeriodLabel,
  formatRangeDate,
} from "./analyticsFormatting";

function AnalyticsDashboard() {
  const initialRange = getPresetRange("this-week");
  const [startDate, setStartDate] = createSignal(initialRange.startDate);
  const [endDate, setEndDate] = createSignal(initialRange.endDate);
  const [activePreset, setActivePreset] =
    createSignal<AnalyticsPresetKey | null>("this-week");
  const [categoryPathOrder, setCategoryPathOrder] =
    createSignal<AnalyticsCategoryPathOrder>("parent-first");

  const hasInvalidRange = createMemo(() => startDate() > endDate());
  const analyticsRequest = createMemo(() => {
    if (hasInvalidRange()) {
      return null;
    }

    return {
      startDate: startDate(),
      endDate: endDate(),
    };
  });
  const [dashboardData] = createResource(analyticsRequest, loadAnalyticsDashboard);

  const activePresetLabel = createMemo(
    () =>
      ANALYTICS_PRESETS.find((preset) => preset.key === activePreset())?.label ??
      "Ручной диапазон",
  );
  const totalHoursMinutes = createMemo(() =>
    formatHoursMinutes(dashboardData()?.totalMinutes ?? 0),
  );
  const chartKind = createMemo<AnalyticsDailyChartKind>(
    () => dashboardData()?.dailyChartKind ?? "bar",
  );

  function applyPreset(presetKey: AnalyticsPresetKey) {
    const nextRange = getPresetRange(presetKey);

    batch(() => {
      setStartDate(nextRange.startDate);
      setEndDate(nextRange.endDate);
      setActivePreset(presetKey);
    });
  }

  function handleStartDateChange(nextStartDate: string) {
    batch(() => {
      setStartDate(nextStartDate);

      if (nextStartDate > endDate()) {
        setEndDate(nextStartDate);
      }

      setActivePreset(null);
    });
  }

  function handleEndDateChange(nextEndDate: string) {
    batch(() => {
      setEndDate(nextEndDate);

      if (nextEndDate < startDate()) {
        setStartDate(nextEndDate);
      }

      setActivePreset(null);
    });
  }

  return (
    <section class="analytics-stack">
      <article class="panel analytics-filter-panel">
        <div class="panel-copy">
          <div>
            <p class="section-label">Период</p>
            <h2>Диапазон отчёта</h2>
          </div>
          <span class="stat-pill">{activePresetLabel()}</span>
        </div>

        <div class="analytics-filter-row">
          <div class="analytics-date-grid">
            <label class="field analytics-date-field">
              <span>Начальная дата</span>
              <input
                class="input-field"
                type="date"
                max={endDate()}
                value={startDate()}
                onInput={(event) => handleStartDateChange(event.currentTarget.value)}
              />
            </label>

            <label class="field analytics-date-field">
              <span>Конечная дата</span>
              <input
                class="input-field"
                type="date"
                min={startDate()}
                value={endDate()}
                onInput={(event) => handleEndDateChange(event.currentTarget.value)}
              />
            </label>
          </div>

          <div class="analytics-preset-block">
            <p class="analytics-preset-title">Быстрые интервалы</p>
            <div class="analytics-preset-grid">
              {ANALYTICS_PRESETS.map((preset) => (
                <button
                  type="button"
                  class="analytics-preset-button"
                  classList={{ "is-active": activePreset() === preset.key }}
                  onClick={() => applyPreset(preset.key)}
                >
                  {preset.label}
                </button>
              ))}
            </div>
          </div>
        </div>
      </article>

      <Show when={hasInvalidRange()}>
        <article class="panel analytics-block-panel">
          <p class="response">Начальная дата не может быть позже конечной.</p>
        </article>
      </Show>

      <Show when={!hasInvalidRange() && dashboardData.error}>
        <article class="panel analytics-block-panel">
          <p class="response">Не удалось загрузить аналитику.</p>
        </article>
      </Show>

      <article class="panel analytics-block-panel">
        <div class="analytics-block-header">
          <div class="analytics-block-heading">
            <div>
              <h2>Суммарное время</h2>
            </div>
          </div>
        </div>

        <div class="analytics-total-card" classList={{ "is-loading": dashboardData.loading }}>
          <p class="analytics-total-label">Общая длительность</p>
          <p class="analytics-total-value">{totalHoursMinutes()}</p>
          <p class="analytics-total-meta">По всем записям за выбранный период.</p>
        </div>
      </article>

      <article class="panel analytics-block-panel">
        <div class="analytics-block-header">
          <div class="analytics-block-heading">
            <div>
              <h2>Структура по категориям и проектам</h2>
            </div>
          </div>
          <div class="analytics-order-controls">
            <button
              type="button"
              class="analytics-order-button"
              classList={{ "is-active": categoryPathOrder() === "parent-first" }}
              onClick={() => setCategoryPathOrder("parent-first")}
            >
              Категория {"->"} Подкатегория
            </button>
            <button
              type="button"
              class="analytics-order-button"
              classList={{ "is-active": categoryPathOrder() === "child-first" }}
              onClick={() => setCategoryPathOrder("child-first")}
            >
              Подкатегория {"->"} Категория
            </button>
          </div>
        </div>

        <AnalyticsCategoryTreeBlock
          nodes={dashboardData()?.categorySummaries ?? []}
          pathOrder={categoryPathOrder()}
        />
      </article>

      <article class="panel analytics-block-panel">
        <div class="analytics-block-header">
          <div class="analytics-block-heading">
            <div>
              <h2>Проекты за период</h2>
            </div>
          </div>
        </div>

        <Show
          when={(dashboardData()?.projectSummaries.length ?? 0) > 0}
          fallback={
            <p class="analytics-empty-state">
              За выбранный диапазон нет тасок с проектами.
            </p>
          }
        >
          <div class="analytics-project-summary-list">
            <For each={dashboardData()?.projectSummaries ?? []}>
              {(project) => (
                <div class="analytics-project-summary">
                  <div class="analytics-project-summary-head">
                    <div>
                      <p class="analytics-project-summary-title">{project.label}</p>
                      <p class="analytics-project-summary-meta">
                        {project.taskCount} таск ·{" "}
                        {project.startDate ? formatRangeDate(project.startDate) : "без даты"}{" "}
                        {"->"}{" "}
                        {project.endDate ? formatRangeDate(project.endDate) : "без даты"}
                      </p>
                    </div>
                    <div class="analytics-project-summary-total">
                      <span>{formatHoursMinutes(project.totalMinutes)}</span>
                      <span>
                        {project.isDone ? "Закрыт" : "Активен"} ·{" "}
                        {formatProjectPeriodLabel(project.finishedInPeriod)}
                      </span>
                    </div>
                  </div>

                  <div class="analytics-project-category-list">
                    <For each={project.categoryBreakdown}>
                      {(category) => (
                        <div class="analytics-project-category-row">
                          <span>{category.label}</span>
                          <span>
                            {category.taskCount} таск ·{" "}
                            {formatHoursMinutes(category.totalMinutes)}
                          </span>
                        </div>
                      )}
                    </For>
                  </div>
                </div>
              )}
            </For>
          </div>
        </Show>
      </article>

      <article class="panel analytics-block-panel">
        <div class="analytics-block-header">
          <div class="analytics-block-heading">
            <div>
              <h2>Доли категорий верхнего уровня</h2>
            </div>
          </div>
        </div>

        <AnalyticsPieChart items={dashboardData()?.rootShareItems ?? []} />
      </article>

      <AnalyticsTrendComparisonBlock
        trendComparison={
          dashboardData()?.trendComparison ?? {
            mode: "custom",
            total: { seriesId: "total", label: "Суммарное время", points: [] },
            categories: [],
          }
        }
      />

      <article class="panel analytics-block-panel">
        <div class="analytics-block-header">
          <div class="analytics-block-heading">
            <div>
              <h2>Динамика по дням</h2>
            </div>
          </div>
        </div>

        <div class="analytics-time-grid">
          <AnalyticsTimeSeriesChart
            title="Часы по дням"
            points={dashboardData()?.dailySeries ?? []}
            chartKind={chartKind()}
            valueKey="totalMinutes"
          />
          <AnalyticsTimeSeriesChart
            title="Количество тасок по дням"
            points={dashboardData()?.dailySeries ?? []}
            chartKind={chartKind()}
            valueKey="taskCount"
          />
        </div>

        <div class="analytics-comments-panel">
          <div class="analytics-mini-heading">
            <p>Комментарии за период</p>
          </div>

          <Show
            when={(dashboardData()?.commentedTasks.length ?? 0) > 0}
            fallback={
              <p class="analytics-empty-state">
                Комментарии к таскам за выбранный период отсутствуют.
              </p>
            }
          >
            <div class="analytics-comment-list">
              <For each={dashboardData()?.commentedTasks ?? []}>
                {(task) => (
                  <div class="analytics-comment-row">
                    <div class="analytics-comment-meta">
                      <span>{formatRangeDate(task.date)}</span>
                      <span>·</span>
                      <span>{task.categoryLabel}</span>
                      <Show when={task.projectLabel}>
                        <span>·</span>
                        <span>{task.projectLabel}</span>
                      </Show>
                      <span>·</span>
                      <span>{formatHoursMinutes(task.timeLength)}</span>
                    </div>
                    <p class="analytics-comment-text">{task.note}</p>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </article>
    </section>
  );
}

export default AnalyticsDashboard;
