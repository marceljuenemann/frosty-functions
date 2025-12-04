import { ComponentFixture, TestBed } from '@angular/core/testing';

import { InvokeFunction } from './invoke-function';

describe('InvokeFunction', () => {
  let component: InvokeFunction;
  let fixture: ComponentFixture<InvokeFunction>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [InvokeFunction]
    })
    .compileComponents();

    fixture = TestBed.createComponent(InvokeFunction);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
