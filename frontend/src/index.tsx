import React from 'react';
import ReactDOM from 'react-dom/client';
import { MotionConfig } from 'framer-motion';
import App from '@app-entry';
import './index.css';

const rootElement = document.getElementById('root');
if (!rootElement) {
  // eslint-disable-next-line no-restricted-syntax -- Fatal initialization error, no recovery possible
  throw new Error("Could not find root element to mount to");
}

const root = ReactDOM.createRoot(rootElement);
root.render(
  <React.StrictMode>
    <MotionConfig reducedMotion="user">
      <App />
    </MotionConfig>
  </React.StrictMode>
);
