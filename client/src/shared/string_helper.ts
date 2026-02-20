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

export function ToConjunctionList(values: string[], conjunction = "and"): string {
    if (values.length == 0) {
        return ""
    }
    if (values.length == 1) {
        return values[0]
    }

    values[values.length-1] = conjunction + " " + values[values.length-1];
    return values.join(', ')
}

