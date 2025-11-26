import { ApplicationConfig, provideBrowserGlobalErrorListeners, provideZonelessChangeDetection } from '@angular/core';
import { provideRouter } from '@angular/router';
import { NGX_MONACO_EDITOR_CONFIG, NgxMonacoEditorConfig } from 'ngx-monaco-editor-v2';

import { routes } from './app.routes';

// TODO: Move into monaco-editor component.
export function onMonacoLoad() {
  const monaco = ((window as any).monaco) as typeof import('monaco-editor');
  // TODO: Load from server.
  const defModel = monaco.editor.createModel(
    `declare module "frosty/fib" {
      /** Calculate the n-th Fibonacci number. */
      export function fib2(n: i32): i32;
    }`,
    'typescript',
    monaco.Uri.parse('file:///frosty.d.ts'),
  );
}

const monacoConfig: NgxMonacoEditorConfig = {
  baseUrl: window.location.origin + `/assets/monaco/min/vs`,
  onMonacoLoad
};

export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    provideZonelessChangeDetection(),
    provideRouter(routes),
    { provide: NGX_MONACO_EDITOR_CONFIG, useValue: monacoConfig }
  ]
};
