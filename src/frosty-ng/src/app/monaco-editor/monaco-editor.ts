import { Component, effect, input, model, signal } from '@angular/core';
import * as monaco from 'monaco-editor';
import { MonacoEditorModule, NgxEditorModel } from 'ngx-monaco-editor-v2';

@Component({
  selector: 'monaco-editor',
  imports: [MonacoEditorModule],
  templateUrl: './monaco-editor.html',
  styleUrl: './monaco-editor.scss',
})
export class MonacoEditor {
  editorOptions = model<monaco.editor.IStandaloneEditorConstructionOptions>({
    theme: 'vs-dark',
    language: 'typescript',
    automaticLayout: true,
    tabSize: 2,
    padding: {
      bottom: 18,
      top: 18
    },
  })

  code = model<string>('')

  model: NgxEditorModel = {
    value: '',
    language: 'typescript',
    uri: monaco.Uri.file('function.ts')
  };

  onEditorInit(editor: monaco.editor.IStandaloneCodeEditor) {
    // Note: Editor content is only set once initialily.
    editor.setValue(this.code());

    editor.onDidChangeModelContent(() => {
      this.code.set(editor.getValue());
    });
  }
}
