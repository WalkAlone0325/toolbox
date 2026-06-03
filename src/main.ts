import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { useThemeStore } from "./stores/theme";
import { useDisplayStore } from "./stores/display";

const app = createApp(App);
const pinia = createPinia();
app.use(pinia);

useThemeStore().applyTheme();
useDisplayStore().applyDisplay();

app.mount("#app");
