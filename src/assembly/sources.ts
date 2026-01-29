// Requires a runtime that loads *.as files as raw text.
import frosty from './frosty/index.as'
import crypto from './frosty/crypto.as'
import env from './frosty/env.as'
import evm from './frosty/evm.as'
import hex from './frosty/hex.as'
import promise from './frosty/promise.as'
import random from './frosty/random.as'
import signer from './frosty/signer.as'

import internal_async from './frosty/internal/async.as'

import runtime from './runtime.as'

/**
 * Source code of all modules.
 */
export const FROSTY_SOURCES = new Map<string, string>([
  ['frosty', frosty],
  ['frosty/crypto', crypto],
  ['frosty/env', env],
  ['frosty/evm', evm],
  ['frosty/hex', hex],
  ['frosty/index', frosty],
  ['frosty/promise', promise],
  ['frosty/random', random],
  ['frosty/signer', signer],
  ['frosty/internal/async', internal_async]
]);

export const RUNTIME_SOURCE = runtime
