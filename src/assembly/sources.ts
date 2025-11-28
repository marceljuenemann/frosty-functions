// Requires a runtime that loads *.as files as raw text.
import frosty from './frosty/index.as'
import evm from './frosty/evm.as'
import promise from './frosty/promise.as'
import random from './frosty/random.as'
import util from './frosty/util.as'

import internal_async from './frosty/internal/async.as'

import runtime from './runtime.as'

/**
 * Source code of all modules.
 */
export const FROSTY_SOURCES = new Map<string, string>([
  ['frosty', frosty],
  ['frosty/evm', evm],
  ['frosty/index', frosty],
  ['frosty/promise', promise],
  ['frosty/random', random],
  ['frosty/util', util],
  ['frosty/internal/async', internal_async],
]);

export const RUNTIME_SOURCE = runtime
