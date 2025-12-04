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

  runFunction() {
    this.signerService.runFrostyFunction(this.function()); // TODO: add params
  }
}
