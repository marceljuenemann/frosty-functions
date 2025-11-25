import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FrostyFunctionEditor } from './frosty-function-editor';

describe('FrostyFunctionEditor', () => {
  let component: FrostyFunctionEditor;
  let fixture: ComponentFixture<FrostyFunctionEditor>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [FrostyFunctionEditor]
    })
    .compileComponents();

    fixture = TestBed.createComponent(FrostyFunctionEditor);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
