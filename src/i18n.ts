import { createI18n } from "vue-i18n";

import en from "./locales/en";
import zh from "./locales/zh";

/** 全局 i18n 实例；locale 由 panelSettings.language 驱动（见 App.vue） */
export const i18n = createI18n({
  legacy: false,
  globalInjection: true,
  locale: "zh",
  fallbackLocale: "zh",
  messages: { zh, en },
});
