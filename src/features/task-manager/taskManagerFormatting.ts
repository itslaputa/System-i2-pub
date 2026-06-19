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

export function formatTaskEntryDate(dateString: string) {
  const [year, month, day] = dateString.split("-");
  const monthIndex = Number(month) - 1;

  if (!year || !month || !day || !SHORT_RUSSIAN_MONTHS[monthIndex]) {
    return dateString;
  }

  return `${day}${SHORT_RUSSIAN_MONTHS[monthIndex]} ${year}`;
}

export function parseTaskDurationToMinutes(durationText: string) {
  const normalizedText = durationText.trim();

  if (!normalizedText) {
    return null;
  }

  if (!normalizedText.includes(".")) {
    return /^\d+$/.test(normalizedText) ? Number(normalizedText) : null;
  }

  const parts = normalizedText.split(".");

  if (parts.length !== 2) {
    return null;
  }

  const [hoursPart, minutesPart] = parts;

  if (!hoursPart || !minutesPart) {
    return null;
  }

  if (!/^\d+$/.test(hoursPart) || !/^\d+$/.test(minutesPart)) {
    return null;
  }

  const hours = Number(hoursPart);
  const minutes = Number(minutesPart);

  return hours * 60 + minutes;
}
