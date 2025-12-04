import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";

// Show window only after React has mounted (prevents white flash)
const showWindow = () => {
  getCurrentWindow().show().catch(console.error);
};

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App onReady={showWindow} />
  </React.StrictMode>,
);
