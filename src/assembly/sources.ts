// Requires a runtime that loads *.as files as raw text.
import frosty from './frosty/index.as'
import promise from './frosty/promise.as'

/**
 * Source code of all modules.
 */
export const FROSTY_SOURCES = {
  'frosty': frosty,
  'frosty/promise': promise,
}
