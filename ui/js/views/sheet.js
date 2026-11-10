// A panel that slides in from the right, over a dimmed window.

const scrim = () => document.querySelector("#scrim");

export class Sheet {
  static all = [];

  /** Takes over `element` and remembers it, so only one sheet is open at a time. */
  constructor(element) {
    this.element = element;
    Sheet.all.push(this);
  }

  /** Whether this sheet is showing. */
  get isOpen() {
    return this.element.classList.contains("open");
  }

  /** Slides the sheet in and closes any other. */
  open() {
    for (const other of Sheet.all) if (other !== this) other.close();
    this.element.classList.add("open");
    this.element.setAttribute("aria-hidden", "false");
    scrim().classList.add("on");
  }

  /** Slides the sheet out. */
  close() {
    this.element.classList.remove("open");
    this.element.setAttribute("aria-hidden", "true");
    if (!Sheet.all.some(sheet => sheet.isOpen)) scrim().classList.remove("on");
  }

  /** Closes every sheet. */
  static closeAll() {
    for (const sheet of Sheet.all) sheet.close();
  }
}
