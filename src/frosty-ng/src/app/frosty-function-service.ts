import { Injectable } from '@angular/core';
import { createActor, frosty_functions_backend, idlFactory } from 'declarations/frosty-functions-backend';
import asc from "assemblyscript/asc";
import { _SERVICE, ExecutionResult, JobRequest, Result_3 } from 'declarations/frosty-functions-backend/frosty-functions-backend.did';
import { Actor, ActorMethodMappedExtended, ActorSubclass, HttpAgent } from '@icp-sdk/core/agent';

export type CompilationResult = {
  success: true
  logs: string
  wasm: Uint8Array<ArrayBufferLike>
  wat: string
} | {
  success: false
  error: string
  logs: string
}

export interface SimulationResult extends ExecutionResult {
  canisterId: string
}

const CANISTER_ID = "uxrrr-q7777-77774-qaaaq-cai";  // Localhost
// const CANISTER_ID = "n6va3-cyaaa-aaaao-qk6pq-cai";  // Production

@Injectable({
  providedIn: 'root',
})
export class FrostyFunctionService {
  private _actor: ActorSubclass<ActorMethodMappedExtended<_SERVICE>> | null = null;

  private async actor(): Promise<ActorSubclass<ActorMethodMappedExtended<_SERVICE>>> {
    if (this._actor) return this._actor;
    let agent = await HttpAgent.create({
      host: 'http://localhost:4943',
      verifyQuerySignatures: false  // TODO: Remove in production
    });
    // Production
    /*
    agent = await HttpAgent.create({
      host: 'https://icp-api.io',
    });
    */
    return this._actor = Actor.createActorWithExtendedDetails<_SERVICE>(
      idlFactory,
      { agent, canisterId: CANISTER_ID }
    )
  }

  /**
   * Compiles the provided function code into a WebAssembly binary.
   */
  // TODO: Make async? Use Web Worker?
  async compile(code: string): Promise<CompilationResult> {
    const stdout = asc.createMemoryStream()

    const outputs = new Map<string, string | Uint8Array>()
    const config: asc.APIOptions = {
      stdout,
      stderr: stdout,
      readFile: (name, basedir) => {
        console.log('readFile', { name, basedir });
        if (name === 'function.ts') {
          return code
        } else if (name === 'node_modules/frosty/fib.ts') {
          // TODO: Include actual API. (ideally at compile-time)
          return `export function fib2(n: i32): i32 {
                    if (n <= 1) return n;
                    return fib2(n - 1) + fib2(n - 2);
                  }
            `;
        }
        return null
      },
      writeFile: (name, contents) => { outputs.set(name, contents) },
      listFiles: () => []
    }
    const options = [
        'function.ts',
        '--textFile', 'function.wat',
        '--outFile',  'function.wasm',
        '--bindings', 'raw'
      ]
    return asc.main(options, config).then(({ error, stdout }): CompilationResult => {
      const output = stdout.toString().trim()
      if (error) {
        return { success: false, error: error.message, logs: output }
      }
      return {
        success: true,
        logs: output,
        wasm: outputs.get('function.wasm') as Uint8Array<ArrayBufferLike>,
        wat: outputs.get('function.wat') as string
      }
    })
  }

  /**
   * Invokes the Frosty Function backend to simulate the function with
   * the given WASM binary.
   */
  async simulate(wasm: Uint8Array): Promise<SimulationResult> {
    // TODO: Configurable request.
    const request: JobRequest = {
      transaction_hash: [],
      block_hash: [],
      data: new Uint8Array(),
      chain: { Evm: { Localhost: null } },
      on_chain_id: [],
      block_number: [],
      function_hash: new Uint8Array(32),
      gas_payment: BigInt(0),
      caller: { EvmAddress: '0x0000000000000000000000000000000000000000' },
    };

    const actor = await this.actor();
    const response = await actor.simulate_execution(request, wasm);
    const result = await response.result;
    if ('Err' in result) {
      throw new Error(`${result.Err}`);
    }
    return { canisterId: CANISTER_ID, ...result.Ok };
  }
}
