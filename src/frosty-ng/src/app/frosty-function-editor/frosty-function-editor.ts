import { Component, HostListener, signal } from '@angular/core';
import { MonacoEditor } from '../monaco-editor/monaco-editor';
import { CompilationResult, DeploymentResult, FrostyFunctionService, SimulationResultExt } from '../frosty-function-service';
import { LogViewer } from '../log-viewer/log-viewer';
import exampleCode from '../../../../assembly/example.as'

type SimulationState =
  { status: 'pending' } |
  { status: 'done', result: SimulationResultExt } |
  { status: 'error', error: Error };


@Component({
  selector: 'frosty-function-editor',
  imports: [MonacoEditor, LogViewer],
  templateUrl: './frosty-function-editor.html',
  styleUrl: './frosty-function-editor.scss',
})
export class FrostyFunctionEditor {
  code = exampleCode;

  compilationResult = signal<CompilationResult | null>(null);
  watUrl: string | null = null
  wasmUrl: string | null = null

  simulation = signal<SimulationState | null>(null)
  deployment = signal<DeploymentResult | null>(null)
  deploying = signal<boolean>(false);

  constructor(private frostyFunctionService: FrostyFunctionService) {}

  async simulate() {
    const compilationResult = await this.compile();
    if (!compilationResult.success) return

    try {
      this.simulation.set({status: 'pending'});
      const result = await this.frostyFunctionService.simulate(compilationResult.wasm)
      this.simulation.set({status: 'done', result});
    } catch (error) {
      error = error instanceof Error ? error : new Error(`${error}`)
      this.simulation.set({status: 'error', error: error as Error});
    }
  }

  async deploy() {
    const compilationResult = await this.compile();
    if (!compilationResult.success) return
    this.deploying.set(true);
    this.deployment.set(await this.frostyFunctionService.deploy({
      binary: compilationResult.wasm,
      source: this.code,
      // TODO: Set to something meaningful.
      compiler: "frosty-ng unstable alpha (client side)"
    }));
    this.deploying.set(false);
  }

  private async compile() {
    this.reset();
    const result = await this.frostyFunctionService.compile(this.code);
    if (result.success) {
      this.watUrl = URL.createObjectURL(new Blob([result.wat], { type: 'text/plain;charset=utf-8' }));
      this.wasmUrl = URL.createObjectURL(new Blob([result.wasm as BlobPart], { type: 'application/wasm' }));
    }
    this.compilationResult.set(result);
    return result;
  }

  private reset() {
    this.compilationResult.set(null);
    if (this.watUrl) {
      URL.revokeObjectURL(this.watUrl);
      this.watUrl = null;
    }
    if (this.wasmUrl) {
      URL.revokeObjectURL(this.wasmUrl);
      this.wasmUrl = null;
    }
    this.simulation.set(null);
    this.deployment.set(null);
    this.deploying.set(false);
  }

  @HostListener('window:keydown', ['$event'])
  handleKeyDown(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && event.key === 's') {
      event.preventDefault();
      this.simulate();
    }
  }

  formatGwei(amount: bigint): string {
    return `${amount / BigInt(1_000_000_000)} gwei`;
  }
}
