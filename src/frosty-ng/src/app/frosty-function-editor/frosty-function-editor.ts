import { Component } from '@angular/core';
import { MonacoEditor } from '../monaco-editor/monaco-editor';
import { CompilationResult, FrostyFunctionService } from '../frosty-function-service';

@Component({
  selector: 'frosty-function-editor',
  imports: [MonacoEditor],
  templateUrl: './frosty-function-editor.html',
  styleUrl: './frosty-function-editor.scss',
})
export class FrostyFunctionEditor {

  compilationResult: CompilationResult | null = null

  constructor(private frostyFunctionService: FrostyFunctionService) {}

  async compile() {
    const code = `
      function main() {
        console.log("Hello world!");
      }`;
    const result = await this.frostyFunctionService.compile(code);
    return this.compilationResult = result;
  }

  async simulate() {
    const result = await this.compile();
  }
}
