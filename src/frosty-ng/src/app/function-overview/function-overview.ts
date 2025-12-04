import { Component, OnInit } from '@angular/core';
import { AsyncPipe, CommonModule } from '@angular/common';
import { ActivatedRoute } from '@angular/router';
import { map, Observable, switchMap } from 'rxjs';
import { FunctionDefinition, FunctionState } from 'declarations/frosty-functions-backend/frosty-functions-backend.did';
import { FrostyFunctionService } from '../frosty-function-service';
import { decodeHex, encodeBase64, encodeHex, formatTimestamp } from '../util';
import { App } from '../app';
import { InvokeFunctionComponent } from '../invoke-function/invoke-function';

@Component({
  selector: 'app-function-overview',
  standalone: true,
  imports: [InvokeFunctionComponent, CommonModule, AsyncPipe],
  templateUrl: './function-overview.html',
  styleUrls: ['./function-overview.scss']
})
export class FunctionOverviewComponent {
  functionId: Observable<FunctionState | null>

  constructor(private route: ActivatedRoute, private service: FrostyFunctionService) {
    this.functionId = this.route.paramMap.pipe(
      map(params => decodeHex(params.get("id")!)),
      switchMap(id => this.service.getFunctionDefinition(id))
    );
  }

  formatTimestamp = formatTimestamp
  encodeHex = encodeHex;
}
