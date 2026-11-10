// Light or dark: Velapor simply follows the system. It stores no preference.

const systemDark = matchMedia("(prefers-color-scheme: dark)");

export class Theme {
  /** The theme the system asks for: "light" or "dark". */
  static current() {
    return systemDark.matches ? "dark" : "light";
  }

  /** Sets data-theme on <html> and tells the charts to pick up the new colours. */
  static paint() {
    const theme = Theme.current();
    if (document.documentElement.dataset.theme === theme) return;
    document.documentElement.dataset.theme = theme;
    document.dispatchEvent(new CustomEvent("themechange", { detail: theme }));
  }

  /** A CSS custom property as a string, for canvas drawing. */
  static token(name) {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  }
}

systemDark.addEventListener("change", () => Theme.paint());
Theme.paint();
