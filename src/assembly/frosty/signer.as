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
      this.derivationPathPtr = changetype<i32>(derivationPath.slice());
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


  // TODO: Move to Address class with a toString method
  /*
  address(): string {
    let buffer = new ArrayBuffer(42 * 2);  // 42 chars * 2 bytes / char
    __evm_caller_wallet_address(changetype<i32>(buffer));
    return String.UTF16.decode(buffer)
  }
  */

  /**
   * Signs the given message according to EIP-191
   * 
   * sign_hash(keccak256(0x19 <0x45 (E)> <thereum Signed Message:\n" + len(message)> <data to sign>))
   * 
   * Use String.UTF8.encode or String.UTF16.encode to convert a string to an ArrayBuffer.
   */
  /*
  signMessage(message: ArrayBuffer): Promise<Uint8Array> {
    let promise = new SharedPromise();
    __evm_caller_wallet_sign_message(changetype<i32>(message), promise.id);
    return promise.map<Uint8Array>(buffer => Uint8Array.wrap(buffer));
  }
    */

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
