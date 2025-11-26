import { Component, HostListener, signal } from '@angular/core';
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
  watUrl: string | null = null
  wasmUrl: string | null = null

  code = 'export function main(): void {\n  console.log("Hello, Frosty!");\n}\n';

  constructor(private frostyFunctionService: FrostyFunctionService) {}

  async compile() {
    const result = await this.frostyFunctionService.compile(this.code);
    if (result.success) {
      if (this.watUrl) URL.revokeObjectURL(this.watUrl);
      if (this.wasmUrl) URL.revokeObjectURL(this.wasmUrl);
      this.watUrl = URL.createObjectURL(new Blob([result.wat], { type: 'text/plain;charset=utf-8' }));
      this.wasmUrl = URL.createObjectURL(new Blob([result.wasm as BlobPart], { type: 'application/wasm' }));
    }
    return this.compilationResult = result;
  }

  async simulate() {
    if (await this.compile() && this.compilationResult?.success) {
      await this.frostyFunctionService.simulate(this.compilationResult.wasm);
    }
  }

  @HostListener('window:keydown', ['$event'])
  handleKeyDown(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && event.key === 's') {
      event.preventDefault();
      this.simulate();
    }
  }
}
