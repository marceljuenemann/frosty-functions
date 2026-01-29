/**
 * Wrapper around a byte array that provides a toString() method in hex encoding.
 */
export class Hex {

  private constructor(public readonly bytes: Uint8Array) {}

  toString(): string {
    return Hex.encode(this.bytes);
  }

  /**
   * Converts a Uint8Array to a hexadecimal string representation.
   */
  static encode(bytes: Uint8Array): string {
    return bytes.reduce((acc, cur) => acc + cur.toString(16).padStart(2, "0"), "0x")
  }

  static wrap(bytes: Uint8Array): Hex {
    return new Hex(bytes);
  }

  static wrapArrayBuffer(bytes: ArrayBuffer): Hex {
    return new Hex(Uint8Array.wrap(bytes));
  }
}
