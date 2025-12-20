export function Title(value: string): string {
    let result = '';
    let shouldCapitalize = true;

    for (const char of value) {
        if (shouldCapitalize && !/\s/.test(char)) {
            result += char.toUpperCase();
        } else {
            result += char;
        }

        shouldCapitalize = /\s/.test(char);
    }

    return result;
}
