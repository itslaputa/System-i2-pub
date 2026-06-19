export function formatHoursMinutes(totalMinutes: number) {
  const safeMinutes = Number.isFinite(totalMinutes) ? Math.max(0, totalMinutes) : 0;
  const hours = Math.floor(safeMinutes / 60);
  const minutes = safeMinutes % 60;

  return `${hours}ч ${String(minutes).padStart(2, "0")}мин`;
}
