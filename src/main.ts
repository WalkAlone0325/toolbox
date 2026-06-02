import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { useThemeStore } from "./stores/theme";

const app = createApp(App);
const pinia = createPinia();
app.use(pinia);

useThemeStore().applyTheme();

app.mount("#app");
