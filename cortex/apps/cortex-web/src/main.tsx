import React from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { registerSW } from 'virtual:pwa-register';
import { App } from "./App";
import "./styles.css";

// Register the service worker only in production.
// In development, Vite HMR already handles refreshes and the SW can make the
// page feel like it is bouncing or reloading while files are in flux.
if (import.meta.env.PROD) {
  registerSW({ immediate: true });
}

createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </React.StrictMode>
);
