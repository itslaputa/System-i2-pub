export type DateRange = {
  startDate: string;
  endDate: string;
};

export type AnalyticsPresetKey =
  | "today"
  | "this-week"
  | "this-month"
  | "this-year"
  | "last-full-week"
  | "last-full-month"
  | "last-full-year";

export type AnalyticsPreset = {
  key: AnalyticsPresetKey;
  label: string;
  getRange: (today: Date) => DateRange;
};

export const ANALYTICS_PRESETS: AnalyticsPreset[] = [
  {
    key: "today",
    label: "Сегодня",
    getRange: (today) => {
      const dateValue = toDateInputValue(today);

      return {
        startDate: dateValue,
        endDate: dateValue,
      };
    },
  },
  {
    key: "this-week",
    label: "За эту неделю",
    getRange: (today) => ({
      startDate: toDateInputValue(startOfWeek(today)),
      endDate: toDateInputValue(endOfWeek(today)),
    }),
  },
  {
    key: "this-month",
    label: "За этот месяц",
    getRange: (today) => ({
      startDate: toDateInputValue(startOfMonth(today)),
      endDate: toDateInputValue(endOfMonth(today)),
    }),
  },
  {
    key: "this-year",
    label: "За этот год",
    getRange: (today) => ({
      startDate: toDateInputValue(startOfYear(today)),
      endDate: toDateInputValue(endOfYear(today)),
    }),
  },
  {
    key: "last-full-week",
    label: "За последнюю полную неделю",
    getRange: (today) => {
      const currentWeekStart = startOfWeek(today);
      const lastWeekStart = shiftDate(currentWeekStart, -7);

      return {
        startDate: toDateInputValue(lastWeekStart),
        endDate: toDateInputValue(shiftDate(lastWeekStart, 6)),
      };
    },
  },
  {
    key: "last-full-month",
    label: "За последний полный месяц",
    getRange: (today) => {
      const startDate = new Date(today.getFullYear(), today.getMonth() - 1, 1);
      const endDate = new Date(today.getFullYear(), today.getMonth(), 0);

      return {
        startDate: toDateInputValue(startDate),
        endDate: toDateInputValue(endDate),
      };
    },
  },
  {
    key: "last-full-year",
    label: "За последний полный год",
    getRange: (today) => {
      const lastYear = today.getFullYear() - 1;

      return {
        startDate: `${lastYear}-01-01`,
        endDate: `${lastYear}-12-31`,
      };
    },
  },
];

export function getPresetRange(presetKey: AnalyticsPresetKey, today = new Date()) {
  const preset = ANALYTICS_PRESETS.find((entry) => entry.key === presetKey);
  return preset?.getRange(today) ?? ANALYTICS_PRESETS[0].getRange(today);
}

function toDateInputValue(date: Date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");

  return `${year}-${month}-${day}`;
}

function shiftDate(date: Date, days: number) {
  const nextDate = new Date(date.getFullYear(), date.getMonth(), date.getDate());
  nextDate.setDate(nextDate.getDate() + days);
  return nextDate;
}

function startOfWeek(date: Date) {
  const normalized = new Date(date.getFullYear(), date.getMonth(), date.getDate());
  const weekOffset = (normalized.getDay() + 6) % 7;
  normalized.setDate(normalized.getDate() - weekOffset);
  return normalized;
}

function endOfWeek(date: Date) {
  return shiftDate(startOfWeek(date), 6);
}

function startOfMonth(date: Date) {
  return new Date(date.getFullYear(), date.getMonth(), 1);
}

function endOfMonth(date: Date) {
  return new Date(date.getFullYear(), date.getMonth() + 1, 0);
}

function startOfYear(date: Date) {
  return new Date(date.getFullYear(), 0, 1);
}

function endOfYear(date: Date) {
  return new Date(date.getFullYear(), 11, 31);
}
