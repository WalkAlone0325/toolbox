import { defineStore } from "pinia";
import { ref, watch } from "vue";

export type FontSize = "small" | "standard" | "large";

export const useDisplayStore = defineStore("display", () => {
  const fontSize = ref<FontSize>(
    (localStorage.getItem("sparkbox.fontSize") as FontSize) || "standard"
  );
  const compact = ref<boolean>(localStorage.getItem("sparkbox.compact") === "1");

  function fontSizePx(): string {
    return { small: "12.5px", standard: "14px", large: "15.5px" }[fontSize.value];
  }

  function applyDisplay() {
    document.documentElement.style.setProperty("--font-base", fontSizePx());
    document.documentElement.style.setProperty(
      "--row-padding-y",
      compact.value ? "6px" : "10px"
    );
    document.documentElement.style.setProperty(
      "--row-gap",
      compact.value ? "2px" : "4px"
    );
    document.documentElement.style.setProperty(
      "--entry-min-height",
      compact.value ? "36px" : "44px"
    );
    if (compact.value) {
      document.documentElement.setAttribute("data-compact", "1");
    } else {
      document.documentElement.removeAttribute("data-compact");
    }
  }

  function setFontSize(size: FontSize) {
    fontSize.value = size;
    localStorage.setItem("sparkbox.fontSize", size);
    applyDisplay();
  }

  function setCompact(v: boolean) {
    compact.value = v;
    localStorage.setItem("sparkbox.compact", v ? "1" : "0");
    applyDisplay();
  }

  watch(fontSize, applyDisplay);
  watch(compact, applyDisplay);

  return { fontSize, compact, setFontSize, setCompact, applyDisplay };
});
