// A short message at the bottom of the window that goes away by itself.

export class Toast {
  /** Takes over `element`. */
  constructor(element) {
    this.element = element;
    this.timer = null;
  }

  /** Shows a message for a few seconds. */
  show(text, kind = "") {
    this.element.textContent = text;
    this.element.className = `toast on ${kind}`;
    clearTimeout(this.timer);
    this.timer = setTimeout(() => this.element.classList.remove("on"), 4200);
  }

  /** Shows an error as a sentence. */
  error(error) {
    this.show(typeof error === "string" ? error : error?.message ?? "Something went wrong.", "error");
  }
}
