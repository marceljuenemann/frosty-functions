import { Component, input } from '@angular/core';
import { FunctionState } from 'declarations/frosty-functions-backend/frosty-functions-backend.did';
import { SignerService } from '../signer';

@Component({
  selector: 'app-invoke-function',
  imports: [],
  templateUrl: './invoke-function.html',
  styleUrl: './invoke-function.scss',
})
export class InvokeFunctionComponent {
  function = input.required<FunctionState>();

  constructor(private signerService: SignerService) {}

  async runFunction() {
    //const chainId = 31337;
    const chainId = 421614;
    const calldata = new Uint8Array([]);  // TODO: configure
    const amount = BigInt(123456);

    const tx = await this.signerService.invokeFrostyFunction(chainId, this.function(), calldata, amount);
    const receipt = await tx.wait();
    console.log('Receipt:', receipt);
    // TODO: Update UI
  }
}
