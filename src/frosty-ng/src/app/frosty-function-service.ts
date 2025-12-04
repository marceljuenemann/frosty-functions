import { Injectable } from '@angular/core';
import { idlFactory } from 'declarations/frosty-functions-backend';
import asc from "assemblyscript/asc";
import { _SERVICE, ExecutionResult, JobRequest, FunctionDefinition, DeployResult } from 'declarations/frosty-functions-backend/frosty-functions-backend.did';
import { Actor, ActorMethodMappedExtended, ActorSubclass, HttpAgent } from '@icp-sdk/core/agent';
import { FROSTY_SOURCES, RUNTIME_SOURCE } from '../../../assembly/sources';

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

export type DeploymentResult = {hash?: string, duplicate?: boolean, error?: string};

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
    await agent.fetchRootKey();  // TODO: Remove in production
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
  async compile(code: string): Promise<CompilationResult> {
    const stdout = asc.createMemoryStream()

    const outputs = new Map<string, string | Uint8Array>()
    const config: asc.APIOptions = {
      stdout,
      stderr: stdout,
      readFile: (name, basedir) => {
        if (name === 'function.ts') {
          return code
        } else if (name === 'runtime.ts') {
          return RUNTIME_SOURCE
        } else if (name.startsWith('node_modules/frosty/')) {
          // Note: name might be 'node_modules/frosty/node_modules/frosty/index.ts'
          const moduleName = 'frosty/' + name.replaceAll('node_modules/frosty/', '').replace('.ts', '')
          if (FROSTY_SOURCES.has(moduleName)) {
            return FROSTY_SOURCES.get(moduleName)!
          }
          console.error("Frosty module not found:", moduleName);
        }
        return null
      },
      writeFile: (name, contents) => { outputs.set(name, contents) },
      listFiles: () => []
    }
    const options = [
        // TODO: Add --memoryLimit (which could only be enforced with server-side compilation though).
        // TODO: Investigate building a build cansiter running JavaScript.
        'runtime.ts',
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
      chain: { Evm: { Localhost: null } },
      transaction_hash: [],
      block_hash: [],
      block_number: [],
      caller: { EvmAddress: '0x0000000000000000000000000000000000000000' },
      function_hash: new Uint8Array(32),
      on_chain_id: [BigInt(42)],
      data: this.parseHex("0xdeadbeef"),
      gas_payment: BigInt(0),
    };

    const actor = await this.actor();
    // TODO: Remove temp_ again
    const response = await actor.temp_simulate_execution(request, wasm);
    const result = await response.result;
    if ('Err' in result) {
      throw new Error(`${result.Err}`);
    }
    return { canisterId: CANISTER_ID, ...result.Ok };
  }

  async deploy(definition: FunctionDefinition): Promise<DeploymentResult> {
    const result = await (await (await this.actor()).deploy(definition)).result;
    if ('Err' in result) {
      return { error: `${result.Err}` };
    } else if ('Duplicate' in result) {
      return { hash: this.encodeBase64(result.Duplicate as Uint8Array), duplicate: true };
    } else if ('Success' in result) {
      return { hash: this.encodeBase64(result.Success as Uint8Array), duplicate: false };
    } else {
      throw new Error("Unknown deploy result");
    }
  }

  // TODO: Use ethers or similar library.
  private parseHex(hex: string): Uint8Array {
    const cleanHex = hex.startsWith('0x') ? hex.slice(2) : hex;
    const padded = cleanHex.length % 2 ? '0' + cleanHex : cleanHex;
    return new Uint8Array(
      padded.match(/.{1,2}/g)!.map(byte => parseInt(byte, 16))
    );
  }

  private encodeHex(bytes: Uint8Array): string {
    return bytes.reduce((acc, cur) => acc + cur.toString(16).padStart(2, "0"), "0x")
  }

  private encodeBase64(bytes: Uint8Array): string {
    // TODO: toBase64 was only introduced in 2025, so need to set target
    return (bytes as any).toBase64({alphabet: "base64url"});
    // return btoa(Array.from(bytes, (byte) => String.fromCodePoint(byte)).join(""))
  }
}
