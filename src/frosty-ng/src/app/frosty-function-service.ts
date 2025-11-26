import { Injectable } from '@angular/core';

@Injectable({
  providedIn: 'root',
})
export class FrostyFunctionService {

  /**
   * Compiles the provided function code into a WebAssembly binary.
   */
  compile(code: string): Uint8Array {
    alert('Compiling function code...');
    return new Uint8Array();
  }

  /**
   * Invokes the Frosty Function backend to simulate the function with
   * the given WASM binary.
   */
  simulate(wasm: Uint8Array): void {
    alert('Simulating function with provided WASM binary...');
  }
}
