import { Component } from '@angular/core';
import * as monaco from 'monaco-editor';
import { MonacoEditorModule, NgxEditorModel } from 'ngx-monaco-editor-v2';

@Component({
  selector: 'monaco-editor',
  imports: [MonacoEditorModule],
  templateUrl: './monaco-editor.html',
  styleUrl: './monaco-editor.scss',
})
export class MonacoEditor {
  editorOptions = {
    theme: 'vs-dark',
    language: 'typescript',
    automaticLayout: true,
    tabSize: 2,
    padding: {
      bottom: 18,
      top: 18
    }
  };
  code: string= 'import {fib2} from "frosty/fib"\n\nfunction x() {\n  console.log("Hello world!");\n}';
  model: NgxEditorModel = {
    value: this.code,
    language: 'typescript',
    uri: monaco.Uri.file('function.ts')
  };
}
