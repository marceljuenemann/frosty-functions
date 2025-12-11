import { Component, input, signal } from '@angular/core';
import { Chain, FunctionState, Job, JobRequest } from 'declarations/frosty-functions-backend/frosty-functions-backend.did';
import { SignerService } from '../signer';
import { TransactionReceipt, TransactionResponse } from 'ethers';
import { FrostyFunctionService } from '../frosty-function-service';
import { AsyncPipe } from '@angular/common';
import { Observable } from 'rxjs';
import { FormControl, FormGroup, FormsModule, ReactiveFormsModule, Validators } from '@angular/forms';
import { decodeHex } from '../util';

const GWEI = BigInt(1_000_000_000);

// TODO: Configurable
const CHAIN: Chain = { Evm: { Localhost: null } };
//  const CHAIN: Chain = { Evm: { ArbitrumSepolia: null } };

// TODO: Move to a config object
export const SCANNER_URL = 'https://sepolia.arbiscan.io';

@Component({
  selector: 'app-invoke-function',
  imports: [ReactiveFormsModule, AsyncPipe],
  templateUrl: './invoke-function.html',
  styleUrl: './invoke-function.scss',
})
export class InvokeFunctionComponent {
  function = input.required<FunctionState>();
  form = new FormGroup({
    calldata: new FormControl('0xdeadbeef', [Validators.pattern(/^0x([a-fA-F0-9][a-fA-F0-9])*$/)]),
    amount: new FormControl(1, [Validators.required, Validators.min(1)])
  });

  // There's three UI states for each step: null (not started), 'pending', and a value (completed).
  transactionId = signal<'pending' | string | null>(null);
  blockNumber = signal<'pending' | string | null>(null);
  job = signal<'pending' | Observable<Job | null> | null>(null);
  error = signal<string | null>(null);

  // TODO: Set based on chain ID
  scannerUrl = signal<string | null>(SCANNER_URL);

  constructor(
    private signerService: SignerService,
    private frostyFunctionService: FrostyFunctionService
  ) {}

  async runFunction() {
    if (this.form.invalid) return;

    this.transactionId.set(null);
    this.blockNumber.set(null);
    this.job.set(null);
    this.error.set(null);

    const tx = await this.submitTransaction();
    // TODO: For ETH mainnet we will want to wait for more confirmations / finalization.
    const receipt = await this.waitForBlockInclusion(tx);
    await this.indexTransaction(receipt);
  }

  private async submitTransaction(): Promise<TransactionResponse> {
    this.transactionId.set('pending');
    try {
      const calldata = decodeHex(this.form.get('calldata')?.value ?? '');
      const amount = BigInt(this.form.get('amount')?.value ?? 0) * GWEI;
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
    this.job.set('pending');
    try {
      const jobRequest = await this.frostyFunctionService.indexTransaction(CHAIN, receipt);
      this.job.set(this.frostyFunctionService.watchJob(CHAIN, Number(jobRequest.on_chain_id[0])));
    } catch (e) {
      this.error.set(`Indexing transaction failed: ${e}`);
      this.job.set(null);
      throw e;
    }
  }

  chainId(chain: Chain): number {
    return this.frostyFunctionService.chainId(chain);
  }

  // TODO: Move somewhere shared.
  jobStatus(job: Job): string {
    return Object.keys(job.status)[0].toLowerCase();
  }
}
