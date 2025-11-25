import { ApplicationConfig, provideBrowserGlobalErrorListeners, provideZonelessChangeDetection } from '@angular/core';
import { provideRouter } from '@angular/router';
import { NGX_MONACO_EDITOR_CONFIG, NgxMonacoEditorConfig } from 'ngx-monaco-editor-v2';

import { routes } from './app.routes';

const monacoConfig: NgxMonacoEditorConfig = {
  baseUrl: window.location.origin + `/assets/monaco/min/vs`,
  defaultOptions: { scrollBeyondLastLine: false }
};

export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    provideZonelessChangeDetection(),
    provideRouter(routes),
    { provide: NGX_MONACO_EDITOR_CONFIG, useValue: monacoConfig }
  ]
};
