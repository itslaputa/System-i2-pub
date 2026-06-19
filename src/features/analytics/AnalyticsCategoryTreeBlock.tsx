import { For, Show, createSignal } from "solid-js";
import type { AnalyticsCategorySummaryNode } from "../../services/tauri/analytics";
import {
  formatHoursMinutes,
  formatProjectPeriodLabel,
} from "./analyticsFormatting";

export type AnalyticsCategoryPathOrder = "parent-first" | "child-first";

type AnalyticsCategoryTreeBlockProps = {
  nodes: AnalyticsCategorySummaryNode[];
  pathOrder: AnalyticsCategoryPathOrder;
};

type CategoryBranchProps = {
  node: AnalyticsCategorySummaryNode;
  depth: number;
  pathOrder: AnalyticsCategoryPathOrder;
  parentLabels: string[];
  isCollapsed: (categoryId: string) => boolean;
  toggleCollapsed: (categoryId: string) => void;
};

function formatPath(labels: string[], pathOrder: AnalyticsCategoryPathOrder) {
  const orderedLabels =
    pathOrder === "parent-first" ? labels : [...labels].reverse();

  return orderedLabels.join(" / ");
}

function CategoryBranch(props: CategoryBranchProps) {
  const nextLabels = [...props.parentLabels, props.node.label];
  const hasChildren = () => props.node.children.length > 0;
  const isCollapsed = () => props.isCollapsed(props.node.categoryId);

  return (
    <div class="analytics-category-branch">
      <div
        class="analytics-category-node"
        style={{ "--analytics-indent": `${props.depth * 18}px` }}
      >
        <div class="analytics-category-head">
          <div class="analytics-category-title-row">
            <Show when={hasChildren()}>
              <button
                type="button"
                class="analytics-collapse-button"
                classList={{ "is-collapsed": isCollapsed() }}
                aria-label={
                  isCollapsed()
                    ? `Развернуть ${props.node.label}`
                    : `Свернуть ${props.node.label}`
                }
                aria-expanded={!isCollapsed()}
                onClick={() => props.toggleCollapsed(props.node.categoryId)}
              >
                {isCollapsed() ? "▸" : "▾"}
              </button>
            </Show>

            <div class="analytics-category-title-copy">
              <p class="analytics-category-label">{props.node.label}</p>
              <Show when={nextLabels.length > 1}>
                <p class="analytics-category-path">
                  {formatPath(nextLabels, props.pathOrder)}
                </p>
              </Show>
              <p class="analytics-category-meta">
                {props.node.taskCount} таск ·{" "}
                {formatHoursMinutes(props.node.totalMinutes)}
              </p>
            </div>
          </div>
          <span class="analytics-category-total">
            {formatHoursMinutes(props.node.totalMinutes)}
          </span>
        </div>

        <Show when={props.node.projects.length > 0}>
          <div class="analytics-category-projects">
            <For each={props.node.projects}>
              {(project) => (
                <div class="analytics-category-project">
                  <span>{project.label}</span>
                  <span>{formatHoursMinutes(project.totalMinutes)}</span>
                  <span>{formatProjectPeriodLabel(project.finishedInPeriod)}</span>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>

      <Show when={hasChildren() && !isCollapsed()}>
        <div class="analytics-category-children">
          <For each={props.node.children}>
            {(child) => (
              <CategoryBranch
                node={child}
                depth={props.depth + 1}
                pathOrder={props.pathOrder}
                parentLabels={nextLabels}
                isCollapsed={props.isCollapsed}
                toggleCollapsed={props.toggleCollapsed}
              />
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}

function AnalyticsCategoryTreeBlock(props: AnalyticsCategoryTreeBlockProps) {
  const [collapsedIds, setCollapsedIds] = createSignal<Set<string>>(new Set());

  function isCollapsed(categoryId: string) {
    return collapsedIds().has(categoryId);
  }

  function toggleCollapsed(categoryId: string) {
    setCollapsedIds((current) => {
      const next = new Set(current);

      if (next.has(categoryId)) {
        next.delete(categoryId);
      } else {
        next.add(categoryId);
      }

      return next;
    });
  }

  return (
    <div class="analytics-category-tree">
      <Show
        when={props.nodes.length > 0}
        fallback={
          <p class="analytics-empty-state">
            За выбранный диапазон нет активных категорий и проектов.
          </p>
        }
      >
        <For each={props.nodes}>
          {(node) => (
            <CategoryBranch
              node={node}
              depth={0}
              pathOrder={props.pathOrder}
              parentLabels={[]}
              isCollapsed={isCollapsed}
              toggleCollapsed={toggleCollapsed}
            />
          )}
        </For>
      </Show>
    </div>
  );
}

export default AnalyticsCategoryTreeBlock;
