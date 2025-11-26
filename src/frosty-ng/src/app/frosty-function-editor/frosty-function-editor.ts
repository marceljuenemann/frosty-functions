import { Component } from '@angular/core';
import { MonacoEditor } from '../monaco-editor/monaco-editor';
import { FrostyFunctionService } from '../frosty-function-service';

@Component({
  selector: 'frosty-function-editor',
  imports: [MonacoEditor],
  templateUrl: './frosty-function-editor.html',
  styleUrl: './frosty-function-editor.scss',
})
export class FrostyFunctionEditor {

  constructor(private frostyFunctionService: FrostyFunctionService) {}

  async simulate() {
    const code = `
      function main() {
        console.log("Hello world!");
      }`;
    const wasm = await this.frostyFunctionService.compile(code);
    alert('Simulating function...');
  }
}
