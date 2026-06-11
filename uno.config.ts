import { defineConfig, presetWind3 } from "unocss";

export default defineConfig({
  presets: [presetWind3()],
  theme: {
    colors: {
      base: "#0e1116",
      card: "#171b22",
      cardHover: "#1d222b",
      line: "#262c36",
      dim: "#8b93a1",
      claude: "#d97757",
      codex: "#19c37d",
    },
  },
});
