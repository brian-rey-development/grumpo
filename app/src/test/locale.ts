const NativeDateTimeFormat = Intl.DateTimeFormat;

/**
 * Makes `Intl.DateTimeFormat` fall back to `locale` when none is given, which stands in for the
 * system locale. The formatting code caches its formatters, so a test file that needs a different
 * locale calls this before its first render and keeps to that locale.
 */
export function installDefaultLocale(locale: string): void {
  class LocalizedDateTimeFormat extends NativeDateTimeFormat {
    constructor(locales: Intl.LocalesArgument = locale, options?: Intl.DateTimeFormatOptions) {
      super(locales, options);
    }
  }
  Intl.DateTimeFormat = LocalizedDateTimeFormat as typeof Intl.DateTimeFormat;
}
