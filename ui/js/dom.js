// Small helpers for building markup safely.

export class Dom {
  /** Escapes text for use inside HTML. */
  static escape(value) {
    return String(value ?? "")
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;")
      .replaceAll("'", "&#39;");
  }

  /** A tagged template that escapes every interpolated value, except `Dom.raw(...)` ones. */
  static html(strings, ...values) {
    return strings.reduce((out, string, i) => {
      if (i === 0) return string;
      const value = values[i - 1];
      const text = Array.isArray(value)
        ? value.map(v => (v instanceof Raw ? v.html : Dom.escape(v))).join("")
        : value instanceof Raw ? value.html : Dom.escape(value);
      return out + text + string;
    }, "");
  }

  /** Marks already-safe HTML so `Dom.html` doesn't escape it. */
  static raw(html) {
    return new Raw(html);
  }

  /** The first element matching `selector`. */
  static $(selector, root = document) {
    return root.querySelector(selector);
  }

  /** Every element matching `selector`, as an array. */
  static $$(selector, root = document) {
    return [...root.querySelectorAll(selector)];
  }
}

/** Markup that is already safe. */
class Raw {
  /** Wraps the markup. */
  constructor(html) { this.html = html; }
}
