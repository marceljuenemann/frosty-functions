import { TestBed } from '@angular/core/testing';

import { FrostyFunctionService } from './frosty-function-service';

describe('FrostyFunctionService', () => {
  let service: FrostyFunctionService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(FrostyFunctionService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
