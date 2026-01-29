import { Promise, Done, DONE } from "./promise";
import { SharedPromise } from "./internal/async";
import { Hex } from "./hex";

const SIGNER_FOR_CALLER = 0;
const SIGNER_FOR_FUNCTION = 1;

/**
 * A signer backed by distributed threshold keys.
 */
export class Signer {
  private derivationPathPtr: i32 = 0;

  private constructor(
    private signerType: i32,
    derivationPath: Uint8Array | null = null
  ) {
    if (derivationPath != null) {
      // Defensive copy into a new ArrayBuffer.
      this.derivationPathPtr = changetype<i32>(derivationPath.slice().buffer);
    }
  }

  /**
   * The ECDSA public key encoded in SEC1 compressed form.
   */
  get publicKey(): Hex {
    let buffer = new ArrayBuffer(33);
    signer_public_key(this.signerType, this.derivationPathPtr, changetype<i32>(buffer));
    return Hex.wrapArrayBuffer(buffer);
  }

  /**
   * The ethereum address controlled by this signer.
   */
  get ethAddress(): Hex {
    let buffer = new ArrayBuffer(20);
    signer_eth_address(this.signerType, this.derivationPathPtr, changetype<i32>(buffer));
    return Hex.wrapArrayBuffer(buffer);
  }

  /**
   * Signs the given message according to EIP-191
   * 
   * sign_hash(keccak256(0x19 <0x45 (E)> <thereum Signed Message:\n" + len(message)> <data to sign>))
   * 
   * Use String.UTF8.encode or String.UTF16.encode to convert a string to an ArrayBuffer.
   */
  signWithEcsda(messageHash: Uint8Array): Promise<Uint8Array> {
    if (messageHash.length != 32) {
      throw new Error(`Message hash must be 32 bytes. Got ${messageHash.length}`);
    }
    let messageHashPtr = changetype<i32>(messageHash.slice().buffer);
    let promise = new SharedPromise();
    sign_with_ecdsa(this.signerType, this.derivationPathPtr, messageHashPtr, promise.id);
    return promise.map<Uint8Array>(buffer => Uint8Array.wrap(buffer));
  }

  /**
   * Creates a signer for the caller of the function. This signer is shared
   * among all Frosty Functions, therefore callers who have assets controlled
   * by this signer should review Frosty Functions carefully before calling them.
   * 
   * The derivationPath can be used to derive different signers.
   */
  static forCaller(derivationPath: Uint8Array | null = null): Signer {
    return new Signer(SIGNER_FOR_CALLER, derivationPath);
  }

  /**
   * Creates a signer for the Frosty Function. This allows Frosty Functions to
   * control assets, but care must be taken to authorize callers appropriately.
   * 
   * The derivationPath can be used to derive different signers, including to
   * derive a signer per caller.
   */
  static forFunction(derivationPath: Uint8Array | null = null): Signer {
    return new Signer(SIGNER_FOR_FUNCTION, derivationPath);
  }
}

@external("❄️", "signer_public_key")
declare function signer_public_key(signerType: i32, derivationPtr: i32, bufferPtr: i32): void;

@external("❄️", "signer_eth_address")
declare function signer_eth_address(signerType: i32, derivationPtr: i32, bufferPtr: i32): void;

@external("❄️", "sign_with_ecdsa")
declare function sign_with_ecdsa(signerType: i32, derivationPtr: i32, messagePtr: i32, promiseId: i32): void;
