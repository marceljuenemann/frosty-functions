import { Component, HostListener, signal } from '@angular/core';
import { MonacoEditor } from '../monaco-editor/monaco-editor';
import { CompilationResult, FrostyFunctionService, SimulationResult } from '../frosty-function-service';

type SimulationState =
  { status: 'pending' } |
  { status: 'done', result: SimulationResult } |
  { status: 'error', error: Error };

@Component({
  selector: 'frosty-function-editor',
  imports: [MonacoEditor],
  templateUrl: './frosty-function-editor.html',
  styleUrl: './frosty-function-editor.scss',
})
export class FrostyFunctionEditor {
  code = 'export function main(): void {\n  console.log("Hello, Frosty!");\n}\n';

  compilationResult: CompilationResult | null = null
  watUrl: string | null = null
  wasmUrl: string | null = null

  simulation = signal<SimulationState | null>(null)

  constructor(private frostyFunctionService: FrostyFunctionService) {}

  async simulate() {
    await this.compile();
    if (!this.compilationResult?.success) return

    try {
      this.simulation.set({status: 'pending'});
      const result = await this.frostyFunctionService.simulate(this.compilationResult.wasm)
      this.simulation.set({status: 'done', result});
    } catch (error) {
      error = error instanceof Error ? error : new Error(`${error}`)
      this.simulation.set({status: 'error', error: error as Error});
      console.log("Simulation error:", error);
    }
  }

  private async compile() {
    this.reset();
    const result = await this.frostyFunctionService.compile(this.code);
    if (result.success) {
      this.watUrl = URL.createObjectURL(new Blob([result.wat], { type: 'text/plain;charset=utf-8' }));
      this.wasmUrl = URL.createObjectURL(new Blob([result.wasm as BlobPart], { type: 'application/wasm' }));
    }
    return this.compilationResult = result;
  }

  private reset() {
    this.compilationResult = null;
    if (this.watUrl) {
      URL.revokeObjectURL(this.watUrl);
      this.watUrl = null;
    }
    if (this.wasmUrl) {
      URL.revokeObjectURL(this.wasmUrl);
      this.wasmUrl = null;
    }
    this.simulation.set(null);
  }

  @HostListener('window:keydown', ['$event'])
  handleKeyDown(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && event.key === 's') {
      event.preventDefault();
      this.simulate();
    }
  }
}
