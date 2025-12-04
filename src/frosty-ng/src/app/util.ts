
// TODO: Use ethers or similar library.
export function decodeHex(hex: string): Uint8Array {
  const cleanHex = hex.startsWith('0x') ? hex.slice(2) : hex;
  const padded = cleanHex.length % 2 ? '0' + cleanHex : cleanHex;
  return new Uint8Array(
    padded.match(/.{1,2}/g)!.map(byte => parseInt(byte, 16))
  );
}

export function encodeHex(bytes: Uint8Array | number[]): string {
  return (bytes as Uint8Array).reduce((acc, cur) => acc + cur.toString(16).padStart(2, "0"), "0x")
}

export function encodeBase64(bytes: Uint8Array | number[]): string {
  // TODO: toBase64 was only introduced in 2025, so need to set target
  return (bytes as any).toBase64({alphabet: "base64url"});
}

export function decodeBase64(encoded: string): Uint8Array {
  // TODO: toBase64 was only introduced in 2025, so need to set target
  return (Uint8Array as any).fromBase64(encoded);
}

export function formatTimestamp(timestamp: bigint): string {
  return new Date(Number(timestamp) / 1_000_000).toISOString().replace('T', ' ').replace('Z', '');
}
