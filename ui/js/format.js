// Numbers written for people: sizes, speeds, shares, durations and times.

export class Format {
  /** Bytes in powers of 1000, the way drives are sold: "18 GB", "512 MB". */
  static bytes(value) {
    const units = ["B", "KB", "MB", "GB", "TB", "PB"];
    let n = Number(value) || 0, i = 0;
    while (n >= 1000 && i < units.length - 1) { n /= 1000; i += 1; }
    const digits = i === 0 || n >= 100 ? 0 : n >= 10 ? 1 : 1;
    return `${n.toFixed(digits).replace(/\.0$/, "")} ${units[i]}`;
  }

  /** Bytes per second: "1.2 MB/s". */
  static rate(value) {
    return `${Format.bytes(value)}/s`;
  }

  /** Bytes per second as megabits per second, the unit internet plans use. */
  static megabits(bytesPerSecond) {
    return (Number(bytesPerSecond) || 0) * 8 / 1_000_000;
  }

  /** A share, 0–100: "42 %". */
  static percent(value, digits = 0) {
    return `${(Number(value) || 0).toFixed(digits)} %`;
  }

  /** A number with a fixed number of digits, or a dash when there is none. */
  static number(value, digits = 0) {
    return value === null || value === undefined || Number.isNaN(value) ? "—" : Number(value).toFixed(digits);
  }

  /** Seconds as "3 d 4 h", "2 h 10 min" or "45 s". */
  static duration(seconds) {
    const s = Math.max(0, Math.round(Number(seconds) || 0));
    const d = Math.floor(s / 86400), h = Math.floor((s % 86400) / 3600), m = Math.floor((s % 3600) / 60);
    if (d) return `${d} d ${h} h`;
    if (h) return `${h} h ${m} min`;
    if (m) return `${m} min`;
    return `${s} s`;
  }

  /** A time of day, "14:02", for a moment in milliseconds. */
  static clock(ms) {
    return new Date(ms).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }

  /** "today 14:02", "Tue 09:10" or "12 Oct" for a moment in milliseconds. */
  static when(ms) {
    const date = new Date(ms), now = new Date();
    if (date.toDateString() === now.toDateString()) return `today ${Format.clock(ms)}`;
    if (now - date < 6 * 86400_000) return `${date.toLocaleDateString([], { weekday: "short" })} ${Format.clock(ms)}`;
    return date.toLocaleDateString([], { day: "numeric", month: "short" });
  }

  /** "since 14:02" or "since Tue 09:10". */
  static since(ms) {
    return `since ${Format.when(ms).replace(/^today /, "")}`;
  }

  /** "1 finding", "3 findings". */
  static plural(count, word, many = `${word}s`) {
    return `${count} ${count === 1 ? word : many}`;
  }
}
