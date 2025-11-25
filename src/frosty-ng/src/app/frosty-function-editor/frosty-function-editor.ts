import { Component } from '@angular/core';
import { MonacoEditor } from '../monaco-editor/monaco-editor';

@Component({
  selector: 'frosty-function-editor',
  imports: [MonacoEditor],
  templateUrl: './frosty-function-editor.html',
  styleUrl: './frosty-function-editor.scss',
})
export class FrostyFunctionEditor {

}
