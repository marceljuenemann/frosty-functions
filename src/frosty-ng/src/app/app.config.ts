import { ApplicationConfig, provideBrowserGlobalErrorListeners, provideZonelessChangeDetection } from '@angular/core';
import { provideRouter } from '@angular/router';
import { NGX_MONACO_EDITOR_CONFIG, NgxMonacoEditorConfig } from 'ngx-monaco-editor-v2';

import { FROSTY_SOURCES } from '../../../assembly/sources';

import { routes } from './app.routes';

// TODO: Move into monaco-editor component.
export async function onMonacoLoad() {
  const monaco = ((window as any).monaco) as typeof import('monaco-editor');
  for (const [module, source] of FROSTY_SOURCES.entries()) {
    monaco.editor.createModel(source, 'typescript', monaco.Uri.file(module + '.ts'));
  }
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
