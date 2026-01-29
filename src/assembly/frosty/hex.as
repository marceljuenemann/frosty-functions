export namespace hex {

  /**
   * Converts a Uint8Array to a hexadecimal string representation.
   */
  export function encode(bytes: Uint8Array): string {
    return bytes.reduce((acc, cur) => acc + cur.toString(16).padStart(2, "0"), "0x")
  }
}
