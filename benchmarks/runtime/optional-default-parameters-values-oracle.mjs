// Independent scenario arithmetic and construction oracle. Does not call optional/default helpers or tested callbacks.
export const ids = ['optional_parameter_int_guard','default_parameter_number_value','default_parameter_boolean_value','default_parameter_string_value','default_parameter_array_value','default_parameter_record_value','default_parameter_dependent_positions','optional_parameter_read_callback','optional_parameter_mutate_callback','default_parameter_owned_reusable_callback','optional_parameter_once_callback','default_parameter_owned_once_callback','default_parameter_stored_family'];
export function validateCase(id, n, seed, input) {
    if (!ids.includes(id) || !Number.isInteger(n) || n < 0 || n > 2048 || !Number.isInteger(seed) || seed < 0 || seed > 1000000 || typeof input !== 'string' || [...input].some(c => { const code = c.codePointAt(0); return code >= 0xd800 && code <= 0xdfff; })) throw new Error('Outside shared scenario domain');
}
function text(input, value, label) { let result = ''; for (let at = 0; at < value % 3 + 1; at++) result += input; return result + ':' + label + ':' + value; }
export function expectedValues(id, n, seed, input) {
    validateCase(id, n, seed, input); const output = []; let mutableState = seed % 11;
    const numbers = [0, -0, 5e-324, -2.5, .5, Infinity, -Infinity, NaN];
    for (let i = 0; i < n; i++) {
        const mode = (i + seed) % 4, value = (i * 13 + seed) % 97, missing = mode < 2;
        if (id === 'optional_parameter_int_guard') output.push(missing ? 1000 : value + 3, 1);
        else if (id === 'default_parameter_number_value') { const useDefault = (i + seed) % 3 < 2; output.push([useDefault ? (value + 100) / 4 : numbers[(i * 13 + seed) % 8], 1, useDefault ? 1 : 0]); }
        else if (id === 'default_parameter_boolean_value') { const isNull = mode === 3, result = value % 3 === 0; output.push(!isNull && result, !isNull && !result, isNull, !missing, missing); }
        else if (id === 'default_parameter_string_value') output.push(text(input, value, missing ? 'default' : 'present'), missing ? '1' : '0');
        else if (id === 'default_parameter_array_value') { const values = []; if (missing) values.push(value, value + 1); else { const sizes = [0, 1, 4, 5, 17]; for (let at = 0; at < sizes[(i * 3 + seed) % 5]; at++) values.push(value + at); } output.push(values, [1, missing ? 1 : 0]); }
        else if (id === 'default_parameter_record_value') { if (mode === 3) output.push(['null']); else { let label = ''; for (let at = 0; at < value % 3 + 1; at++) label += input; output.push(['record', String(value + (missing ? 100 : 0)), label + (missing ? ':default' : ':present')]); } output.push(['1', missing ? '1' : '0']); }
        else if (id === 'default_parameter_dependent_positions') { const required = value % 7, first = mode === 0 || mode === 3 ? value + 10 : value, second = mode < 2 ? first + required + 1 : value + 20; output.push([first, required, second, first + second], [1, mode === 0 ? 2 : mode === 2 ? 0 : 1]); }
        else if (id === 'optional_parameter_read_callback') output.push(missing ? [] : [value + seed % 11 + 3], [missing ? 0 : 1, 0]);
        else if (id === 'optional_parameter_mutate_callback') { const values = []; if (!missing) { mutableState++; values.push(value + mutableState); } output.push(values, [missing ? 0 : 1, 0]); }
        else if (id === 'default_parameter_owned_reusable_callback') output.push([2 * value + (missing ? 101 : 1), 2 * value + (missing ? 103 : 3)], [2, missing ? 1 : 0]);
        else if (id === 'optional_parameter_once_callback') output.push(missing ? [] : [text(input, value, 'present')], [missing ? '0' : '1', '0']);
        else if (id === 'default_parameter_owned_once_callback') output.push([text(input, value, missing ? 'default' : 'present')], ['1', missing ? '1' : '0']);
        else if (id === 'default_parameter_stored_family') { const first = (i + seed) % 2 === 0; output.push([(missing ? first ? 10 : 100 : value) + (first ? 1 : 2)], [1, missing ? 1 : 0]); }
    }
    return output;
}
