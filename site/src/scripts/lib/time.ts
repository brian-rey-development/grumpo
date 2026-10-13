const SECONDS_PER_MINUTE = 60;
const SECONDS_PER_HOUR = 3_600;

const twoDigits = (value: number): string => String(value).padStart(2, '0');

export function formatClock(totalSeconds: number): string {
  const seconds = Math.max(0, Math.floor(totalSeconds));
  const hours = Math.floor(seconds / SECONDS_PER_HOUR);
  const minutes = Math.floor((seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE);
  const rest = twoDigits(seconds % SECONDS_PER_MINUTE);
  return hours > 0 ? `${hours}:${twoDigits(minutes)}:${rest}` : `${minutes}:${rest}`;
}

export function minutesToSeconds(minutes: number): number {
  return minutes * SECONDS_PER_MINUTE;
}
