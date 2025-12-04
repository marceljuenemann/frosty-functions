import { Component, OnInit } from '@angular/core';
import { AsyncPipe, CommonModule } from '@angular/common';
import { ActivatedRoute } from '@angular/router';
import { map, Observable } from 'rxjs';
import { FunctionDefinition } from 'declarations/frosty-functions-backend/frosty-functions-backend.did';

@Component({
  selector: 'app-function-overview',
  standalone: true,
  imports: [CommonModule, AsyncPipe],
  templateUrl: './function-overview.html',
  styleUrls: ['./function-overview.scss']
})
export class FunctionOverviewComponent {
  functionId: Observable<string>

  constructor(private route: ActivatedRoute) {
    this.functionId = this.route.paramMap.pipe(
      map(params => params.get("id")!),
    );

  }
}
