import { Component, signal } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { NgbAlert, NgbModule } from '@ng-bootstrap/ng-bootstrap';

@Component({
  selector: 'app-root',
  imports: [RouterOutlet, NgbAlert],
  templateUrl: './app.html',
  styleUrl: './app.scss'
})
export class App {
  protected readonly title = signal('frosty-ng');
}
