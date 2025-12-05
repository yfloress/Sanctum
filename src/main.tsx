import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App.tsx";

// Hide the HTML splash screen with a fade animation
const hideSplashScreen = () => {
  const splash = document.getElementById("splash-screen");
  if (splash) {
    splash.classList.add("hidden");
    // Remove from DOM after transition completes
    setTimeout(() => splash.remove(), 300);
  }
};

// Show window and hide splash after React has mounted
const onReady = () => {
  getCurrentWindow().show().catch(console.error);
  hideSplashScreen();
};

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App onReady={onReady} />
  </React.StrictMode>,
);
