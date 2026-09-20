// DiskAnalyzer — renderer entry point.
// Per 08-RENDERER-REACT-APP.md §8.2.

import React from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import './styles/globals.css';

const root = createRoot(document.getElementById('root')!);
root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

// Type-safety: declare the global window.diskanalyzer.
declare global {
  interface Window {
    diskanalyzer: import('@disk-analyzer/ipc').DiskAnalyzerAPI;
  }
}
