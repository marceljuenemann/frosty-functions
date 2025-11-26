import { Component, input } from '@angular/core';
import { LogEntry } from 'declarations/frosty-functions-backend/frosty-functions-backend.did';

@Component({
  selector: 'app-log-viewer',
  imports: [],
  templateUrl: './log-viewer.html',
  styleUrl: './log-viewer.scss',
})
export class LogViewer {

  logs = input<Array<LogEntry>>([]);

}
