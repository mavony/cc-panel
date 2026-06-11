import { onMounted, onUnmounted, shallowRef } from "vue";

/** 每秒跳动的当前时间戳（毫秒），用于相对时间显示 */
export function useNow(intervalMs = 1000) {
  const now = shallowRef(Date.now());
  let timer: ReturnType<typeof setInterval> | undefined;

  onMounted(() => {
    timer = setInterval(() => {
      now.value = Date.now();
    }, intervalMs);
  });
  onUnmounted(() => clearInterval(timer));

  return now;
}
