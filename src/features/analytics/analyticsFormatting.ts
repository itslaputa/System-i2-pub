import { formatHoursMinutes as formatSharedHoursMinutes } from "../../utils/formatDuration";

const SHORT_RUSSIAN_MONTHS = [
  "янв",
  "фев",
  "мар",
  "апр",
  "май",
  "июн",
  "июл",
  "авг",
  "сен",
  "окт",
  "ноя",
  "дек",
];

export function formatRangeDate(dateValue: string) {
  const [year, month, day] = dateValue.split("-");
  const monthIndex = Number(month) - 1;

  if (!year || !month || !day || !SHORT_RUSSIAN_MONTHS[monthIndex]) {
    return dateValue;
  }

  return `${Number(day)} ${SHORT_RUSSIAN_MONTHS[monthIndex]} ${year}`;
}

export function formatHoursMinutes(totalMinutes: number) {
  const safeMinutes = Number.isFinite(totalMinutes) ? Math.max(0, totalMinutes) : 0;

  if (safeMinutes < 60) {
    return `${safeMinutes}мин`;
  }

  return formatSharedHoursMinutes(safeMinutes);
}

export function formatProjectPeriodLabel(finishedInPeriod: boolean) {
  return finishedInPeriod ? "Закончен за период" : "Был в работе";
}
