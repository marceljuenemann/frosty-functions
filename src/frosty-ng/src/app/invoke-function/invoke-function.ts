import { Component, input } from '@angular/core';
import { FunctionState } from 'declarations/frosty-functions-backend/frosty-functions-backend.did';

@Component({
  selector: 'app-invoke-function',
  imports: [],
  templateUrl: './invoke-function.html',
  styleUrl: './invoke-function.scss',
})
export class InvokeFunctionComponent {
  function = input.required<FunctionState>();

  runFunction() {
    console.log("Running function:", this.function());
  }
}
