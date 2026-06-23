import { useEffect, useState } from "react";
import logo from "./assets/logo.svg";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";

function App() {
  return (
    <main className="container">
      <h1>FUNNY MIC!</h1>

      <div className="row">
        <img src={logo} className="logo" />
      </div>
      <p>oh also there's a logo now</p>
    </main>
  );
}

export default App;
