import { Component, signal } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { NgbAlert, NgbModule } from '@ng-bootstrap/ng-bootstrap';
import { FrostyFunctionEditor } from './frosty-function-editor/frosty-function-editor';

@Component({
  selector: 'app-root',
  imports: [RouterOutlet, NgbAlert, FrostyFunctionEditor],
  templateUrl: './app.html',
  styleUrl: './app.scss'
})
export class App {
  protected readonly title = signal('frosty-ng');
}
