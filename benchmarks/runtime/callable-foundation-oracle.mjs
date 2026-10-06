// Independent arithmetic specifications for callable ownership workloads.
export const callableFoundationIds = Object.freeze([
  'owned_callable_fields', 'annotated_once_locals',
  'default_conditional_callable_fields', 'once_factory_results',
  'generic_function_record_pipeline', 'callable_array_mutable_pipeline', 'callable_array_transfer_pipeline', 'callable_array_once_transfer', 'callable_array_reduction_pipeline',
]);
export function expectedCallableFoundation(id, n, seed) {
  const series = n * (n - 1) / 2;
  switch (id) {
    case 'owned_callable_fields': return seed + 2 * n * seed + 2 * series;
    case 'annotated_once_locals': return seed + n * seed + 2 * series;
    case 'callable_array_mutable_pipeline': return seed + 6 * n * seed + 6 * series + 19 * n;
    case 'callable_array_transfer_pipeline': return seed + 4 * n * seed + 4 * series + 12 * n;
    case 'callable_array_once_transfer': return seed + 4 * n * seed + 4 * series + 10 * n;
    case 'callable_array_reduction_pipeline': return seed + 15 * n * seed + 15 * series + 54 * n;
    case 'once_factory_results': return seed + n * seed + series;
    case 'default_conditional_callable_fields': {
      const even = Math.ceil(n / 2), thirds = Math.ceil(n / 3);
      return seed + n * seed + 2 * even * (even - 1)
        + n * (n + 1) / 2 + 3 * thirds * (thirds - 1) / 2 - thirds;
    }
    case 'generic_function_record_pipeline': {
      let result = seed;
      for (let i = 0; i < n; i++) result += (i + seed) % 1009;
      return result;
    }
    default: throw new Error(`No callable foundation oracle for ${id}`);
  }
}
