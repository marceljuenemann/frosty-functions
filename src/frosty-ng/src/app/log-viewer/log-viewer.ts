import { DatePipe } from '@angular/common';
import { Component, input } from '@angular/core';
import { Commit, LogEntry } from 'declarations/frosty-functions-backend/frosty-functions-backend.did';
import { formatTimestamp } from '../util';

@Component({
  selector: 'app-log-viewer',
  imports: [],
  templateUrl: './log-viewer.html',
  styleUrl: './log-viewer.scss',
})
export class LogViewer {

  commits = input<Array<Commit>>([]);

  formatTimestamp = formatTimestamp;

  toGwei(fee: bigint): number {
    return Number(fee / BigInt(1_000_000_000));
  }
}
