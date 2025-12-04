import { Routes } from '@angular/router';
import { FunctionOverviewComponent } from './function-overview/function-overview';
import { FrostyFunctionEditor } from './frosty-function-editor/frosty-function-editor';

export const routes: Routes = [
  {
    path: '',
    component: FrostyFunctionEditor
  },
  {
    path: 'functions/:id',
    component: FunctionOverviewComponent,
  },
];
