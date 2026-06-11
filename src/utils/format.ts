/** 相对时间，如 "刚刚" / "3 分钟前" */
export function formatRelative(epochMs: number, nowMs: number): string {
  const diff = Math.max(0, nowMs - epochMs);
  const sec = Math.floor(diff / 1000);
  if (sec < 60) return "刚刚";
  const min = Math.floor(sec / 60);
  if (min < 60) return `${min} 分钟前`;
  const hour = Math.floor(min / 60);
  if (hour < 24) return `${hour} 小时前`;
  return `${Math.floor(hour / 24)} 天前`;
}

/** 重置倒计时，如 "2 小时后重置" */
export function formatResetAt(epochSec: number | null, nowMs: number): string {
  if (!epochSec) return "";
  const diff = epochSec * 1000 - nowMs;
  if (diff <= 0) return "即将重置";
  const min = Math.ceil(diff / 60000);
  if (min < 60) return `${min} 分钟后重置`;
  const hour = Math.floor(min / 60);
  if (hour < 48) return `${hour} 小时后重置`;
  return `${Math.floor(hour / 24)} 天后重置`;
}

export function baseName(path: string): string {
  const i = path.lastIndexOf("/");
  return i >= 0 ? path.slice(i + 1) : path;
}
