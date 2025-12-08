import { Component, input, signal } from '@angular/core';
import { Chain, FunctionState, JobRequest } from 'declarations/frosty-functions-backend/frosty-functions-backend.did';
import { SignerService } from '../signer';
import { TransactionReceipt, TransactionResponse } from 'ethers';
import { FrostyFunctionService } from '../frosty-function-service';

// TODO: Configurable
const CHAIN: Chain = { Evm: { Localhost: null } };

// TODO: Move to a config object
export const SCANNER_URL = 'https://sepolia.arbiscan.io';

@Component({
  selector: 'app-invoke-function',
  imports: [],
  templateUrl: './invoke-function.html',
  styleUrl: './invoke-function.scss',
})
export class InvokeFunctionComponent {
  function = input.required<FunctionState>();

  // There's three UI states for each step: null (not started), 'pending', and a value (completed).
  transactionId = signal<'pending' | string | null>(null);
  blockNumber = signal<'pending' | string | null>(null);
  jobId = signal<'pending' | { chain: Chain, id: bigint } | null>(null);
  error = signal<string | null>(null);

  // TODO: Set based on chain ID
  scannerUrl = signal<string | null>(SCANNER_URL);

  constructor(
    private signerService: SignerService,
    private frostyFunctionService: FrostyFunctionService
  ) {}

  async runFunction() {
    this.transactionId.set(null);
    this.blockNumber.set(null);
    this.jobId.set(null);
    this.error.set(null);

    const tx = await this.submitTransaction();
    // TODO: For ETH mainnet we will want to wait for more confirmations / finalization.
    const receipt = await this.waitForBlockInclusion(tx);
    await this.indexTransaction(receipt);
  }

  private async submitTransaction(): Promise<TransactionResponse> {
    this.transactionId.set('pending');
    try {
      const calldata = new Uint8Array([]);  // TODO: configure
      const amount = BigInt(123456);
      const chainId = this.frostyFunctionService.chainId(CHAIN);
      const tx = await this.signerService.invokeFrostyFunction(chainId, this.function(), calldata, amount);
      this.transactionId.set(tx.hash);
      return tx;
    } catch (e) {
      this.error.set(`Transaction submission failed: ${e}`);
      this.transactionId.set(null);
      throw e;
    }
  }

  private async waitForBlockInclusion(tx: TransactionResponse): Promise<TransactionReceipt> {
    this.blockNumber.set('pending');
    try {
      const receipt = await tx.wait();
      if (receipt == null) {
        this.error.set('Transaction failed: no receipt');
        this.blockNumber.set(null);
        throw new Error("No receipt");
      }
      this.blockNumber.set(receipt.blockNumber.toString());
      return receipt;
    } catch (e) {
      this.error.set(`Waiting for block inclusion failed: ${e}`);
      this.blockNumber.set(null);
      throw e;
    }
  }

  private async indexTransaction(receipt: TransactionReceipt): Promise<void> {
    this.jobId.set('pending');
    try {
      const jobRequest = await this.frostyFunctionService.indexTransaction(CHAIN, receipt);
      this.jobId.set({ chain: CHAIN, id: jobRequest.on_chain_id[0]! });
    } catch (e) {
      this.error.set(`Indexing transaction failed: ${e}`);
      this.jobId.set(null);
      throw e;
    }
  }

  chainId(chain: Chain): number {
    return this.frostyFunctionService.chainId(chain);
  }
}
