import { i18n } from "../i18n";

const t = (key: string, n: number) => i18n.global.t(key, { n });

/** 相对时间，如 "刚刚" / "3 分钟前" */
export function formatRelative(epochMs: number, nowMs: number): string {
  const diff = Math.max(0, nowMs - epochMs);
  const sec = Math.floor(diff / 1000);
  if (sec < 60) return i18n.global.t("rel.justNow");
  const min = Math.floor(sec / 60);
  if (min < 60) return t("rel.minAgo", min);
  const hour = Math.floor(min / 60);
  if (hour < 24) return t("rel.hourAgo", hour);
  return t("rel.dayAgo", Math.floor(hour / 24));
}

/** 重置倒计时，如 "2 小时后重置" */
export function formatResetAt(epochSec: number | null, nowMs: number): string {
  if (!epochSec) return "";
  const diff = epochSec * 1000 - nowMs;
  if (diff <= 0) return i18n.global.t("rel.resetSoon");
  const min = Math.ceil(diff / 60000);
  if (min < 60) return t("rel.resetMin", min);
  const hour = Math.floor(min / 60);
  if (hour < 48) return t("rel.resetHour", hour);
  return t("rel.resetDay", Math.floor(hour / 24));
}

export function baseName(path: string): string {
  const i = path.lastIndexOf("/");
  return i >= 0 ? path.slice(i + 1) : path;
}
