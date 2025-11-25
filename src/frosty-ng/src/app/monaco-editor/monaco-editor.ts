import { Component } from '@angular/core';
import { Uri } from 'monaco-editor';
import { MonacoEditorModule, NgxEditorModel } from 'ngx-monaco-editor-v2';

@Component({
  selector: 'monaco-editor',
  imports: [MonacoEditorModule],
  templateUrl: './monaco-editor.html',
  styleUrl: './monaco-editor.scss',
})
export class MonacoEditor {
  editorOptions = {theme: 'vs-dark', language: 'javascript'};
  code: string= 'function x() {\nconsole.log("Hello world!");\n}';
  model: NgxEditorModel = {
    value: this.code,
    language: 'typescript',
    uri: Uri.parse('file://function.ts')
  };
}
