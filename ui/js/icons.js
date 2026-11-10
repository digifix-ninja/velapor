// Every icon the window uses, as inline SVG drawn with currentColor.

const stroke = d => `<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${d}</svg>`;

export const Icons = {
  preflight: stroke('<path d="M2 9.5 14 3l-3.5 11-2.2-4.6L2 9.5Z"/>'),
  info: stroke('<circle cx="8" cy="8" r="6.2"/><path d="M8 7.2v4"/><circle cx="8" cy="4.9" r=".5" fill="currentColor"/>'),
  close: stroke('<path d="m4 4 8 8M12 4l-8 8"/>'),
  speed: stroke('<path d="M2.5 11a5.5 5.5 0 1 1 11 0"/><path d="m8 11 3-3.5"/>'),

  // Severities: an icon each, always shown with its colour and its word
  severity: {
    clear: stroke('<path d="m3.5 8.3 2.8 2.8 6.2-6.4"/>'),
    notice: stroke('<circle cx="8" cy="8" r="6"/><path d="M8 7.3v3.6M8 5v.1"/>'),
    warning: stroke('<path d="M8 2.2 14.5 13.5h-13L8 2.2Z"/><path d="M8 6.5v3M8 11.6v.1"/>'),
    critical: stroke('<path d="M5.4 1.8h5.2l3.6 3.6v5.2l-3.6 3.6H5.4l-3.6-3.6V5.4z"/><path d="M8 5v3.8M8 11v.1"/>'),
    unknown: stroke('<circle cx="8" cy="8" r="6"/><path d="M6.4 6.2a1.7 1.7 0 1 1 2.3 1.6c-.5.2-.7.6-.7 1.1v.3M8 11.4v.1"/>'),
  },
};

/** The words that go with each severity. */
export const SEVERITY_WORD = { clear: "Clear", notice: "Notice", warning: "Warning", critical: "Critical" };
