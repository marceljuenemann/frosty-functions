// Requires a runtime that loads *.as files as raw text.
import frosty from './frosty/index.as'
import promise from './frosty/promise.as'
import internal from './frosty/internal.as'
import internal_async from './frosty/internal/async.as'

/**
 * Source code of all modules.
 */
export const FROSTY_SOURCES = new Map<string, string>([
  ['frosty', frosty],
  ['frosty/index', frosty],
  ['frosty/promise', promise],
  ['frosty/internal', internal],
  ['frosty/internal/async', internal_async],
]);
