import { Component, signal } from '@angular/core';
import { MonacoEditor } from '../monaco-editor/monaco-editor';
import { CompilationResult, FrostyFunctionService } from '../frosty-function-service';
import { NgxEditorModel } from 'ngx-monaco-editor-v2';
import * as monaco from 'monaco-editor';

@Component({
  selector: 'frosty-function-editor',
  imports: [MonacoEditor],
  templateUrl: './frosty-function-editor.html',
  styleUrl: './frosty-function-editor.scss',
})
export class FrostyFunctionEditor {

  compilationResult: CompilationResult | null = null

  code = 'export function main(): void {\n  console.log("Hello, Frosty!");\n}\n';

  constructor(private frostyFunctionService: FrostyFunctionService) {}

  async compile() {
    console.log('Compiling code:', this.code);
    const code = this.code;
    const result = await this.frostyFunctionService.compile(code);
    return this.compilationResult = result;
  }

  async simulate() {
    const result = await this.compile();
  }
}
