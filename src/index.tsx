/* @refresh reload */
import { render } from "solid-js/web";

import "./styles.css";
import App from "./App";
import { Router } from '@solidjs/router';
import { LanguageProvider } from './i18n/index';

// Remove any debug/inspector borders that might be added by browser tools
const removeDebugBorders = () => {
  const style = document.createElement('style');
  style.textContent = `
    * {
      border: none !important;
      outline: none !important;
    }

    *[style*="border: 1px solid red"],
    *[style*="border: 2px solid red"],
    *[style*="border: 1px solid rgb(255, 0, 0)"],
    *[style*="border: 2px solid rgb(255, 0, 0)"],
    *[style*="border: 1px solid #ff0000"],
    *[style*="border: 2px solid #ff0000"] {
      border: none !important;
    }

    /* Override any inspector styles */
    .__web-inspector-hide-shortcut__,
    *[data-inspector],
    *[data-debug],
    *[class*="inspector"],
    *[class*="debug"] {
      border: none !important;
      outline: none !important;
      box-shadow: none !important;
    }
  `;
  document.head.appendChild(style);

  // Also remove any existing red borders
  const removeRedBorders = () => {
    const elements = document.querySelectorAll('*');
    elements.forEach(el => {
      const computedStyle = window.getComputedStyle(el);
      if (computedStyle.border.includes('red') ||
        computedStyle.borderColor.includes('red') ||
        computedStyle.borderColor.includes('rgb(255, 0, 0)')) {
        (el as HTMLElement).style.border = 'none';
        (el as HTMLElement).style.outline = 'none';
      }
    });
  };

  // Run immediately and on DOM changes
  removeRedBorders();
  const observer = new MutationObserver(removeRedBorders);
  observer.observe(document.body, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ['style', 'class']
  });
};

// Run after DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', removeDebugBorders);
} else {
  removeDebugBorders();
}

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
