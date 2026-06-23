import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { getCurrentWindow } from "@tauri-apps/api/window";

// just hide the window when closing it.
// this lets us run in the background and exit from the tray!
let window = getCurrentWindow();
let unlisten = await window.onCloseRequested(async (event) => {
  event.preventDefault();
  window.hide();
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
