import { TestBed } from '@angular/core/testing';

import { FrostyFunction } from './frosty-function';

describe('FrostyFunction', () => {
  let service: FrostyFunction;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(FrostyFunction);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
