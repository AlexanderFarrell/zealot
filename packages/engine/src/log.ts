type SafeContext = Record<string, string | number>;

function formatContext(context?: SafeContext): string {
    if (!context || Object.keys(context).length === 0) return '';
    return ' ' + JSON.stringify(context);
}

/**
 * Log an informational message.
 * context values must be IDs or status codes only — never item content, titles, or attribute values.
 */
export function logInfo(message: string, context?: SafeContext): void {
    console.info(`[zealot] ${message}${formatContext(context)}`);
}

/**
 * Log an error.
 * context values must be IDs or status codes only — never item content, titles, or attribute values.
 */
export function logError(message: string, context?: SafeContext): void {
    console.error(`[zealot] ${message}${formatContext(context)}`);
}
