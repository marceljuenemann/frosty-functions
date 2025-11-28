import { DatePipe } from '@angular/common';
import { Component, input } from '@angular/core';
import { Commit, LogEntry } from 'declarations/frosty-functions-backend/frosty-functions-backend.did';

@Component({
  selector: 'app-log-viewer',
  imports: [],
  templateUrl: './log-viewer.html',
  styleUrl: './log-viewer.scss',
})
export class LogViewer {

  commits = input<Array<Commit>>([]);

  timestampToDate(timestamp: bigint): string {
    return new Date(Number(timestamp) / 1_000_000).toISOString().replace('T', ' ').replace('Z', '');
  }
}
