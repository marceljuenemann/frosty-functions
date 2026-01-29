/**
 * Keccak-256 hash of the given data.
 */
export function keccak256(data: Uint8Array): Uint8Array {
  let buffer = new ArrayBuffer(32);
  crypto_keccak256(changetype<i32>(data.slice().buffer), changetype<i32>(buffer));
  return Uint8Array.wrap(buffer);
}

@external("❄️", "crypto_keccak256")
declare function crypto_keccak256(dataPtr: i32, bufferPtr: i32): void;
