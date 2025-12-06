import { Component, input, signal } from '@angular/core';
import { FunctionState } from 'declarations/frosty-functions-backend/frosty-functions-backend.did';
import { SignerService } from '../signer';
import { TransactionReceipt, TransactionResponse } from 'ethers';

// TODO: Configurable
const CHAIN_ID = 31337;
// const chainId = 421614;


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
  jobId = signal<'pending' | number | null>(null);
  error = signal<string | null>(null);

  // TODO: Set based on chain ID
  scannerUrl = signal<string | null>('https://sepolia.arbiscan.io');

  constructor(private signerService: SignerService) {}

  async runFunction() {
    this.transactionId.set(null);
    this.blockNumber.set(null);
    this.jobId.set(null);
    this.error.set(null);

    const tx = await this.submitTransaction();
    const receipt = await this.waitForBlockInclusion(tx);

  }

  private async submitTransaction(): Promise<TransactionResponse> {
    this.transactionId.set('pending');
    try {
      const calldata = new Uint8Array([]);  // TODO: configure
      const amount = BigInt(123456);
      const tx = await this.signerService.invokeFrostyFunction(CHAIN_ID, this.function(), calldata, amount);
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

}
