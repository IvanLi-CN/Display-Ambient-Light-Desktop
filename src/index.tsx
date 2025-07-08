/* @refresh reload */
import { render } from "solid-js/web";

import "./styles.css";
import App from "./App";
import { Router } from '@solidjs/router';
import { LanguageProvider } from './i18n/index';

render(
  () => (
    <LanguageProvider>
      <Router>
        <App />
      </Router>
    </LanguageProvider>
  ),
  document.getElementById('root') as HTMLElement,
);
