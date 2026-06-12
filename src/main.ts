import { createPinia } from "pinia";
import { createApp } from "vue";

import "virtual:uno.css";
import "./assets/main.css";
import App from "./App.vue";
import { i18n } from "./i18n";

createApp(App).use(createPinia()).use(i18n).mount("#app");
