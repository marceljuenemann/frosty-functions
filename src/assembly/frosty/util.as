
/**
 * Converts a Uint8Array to a hexadecimal string representation.
 */
export function toHexString(bytes: Uint8Array): string {
  return bytes.reduce((acc, cur) => acc + cur.toString(16), "0x")
}
