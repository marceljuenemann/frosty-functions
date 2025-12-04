import { TestBed } from '@angular/core/testing';

import { Signer } from './signer';

describe('Signer', () => {
  let service: Signer;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(Signer);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
