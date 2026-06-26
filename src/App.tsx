import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";
import { RefreshCcw } from "lucide-react";
import KeyComboButton from "./components/KeyComboButton";
import { register, unregisterAll } from "@tauri-apps/plugin-global-shortcut";

// const asleep = (ms: number) =>
//   new Promise((resolve) => setTimeout(resolve, ms));

function App() {
  const [sources, setSources] = useState<Record<string, string>>({});
  const [currentDevice, setCurrent] = useState<string>("");

  async function getSources() {
    setSources(await invoke("get_device_list"));
    setCurrent(await invoke("get_swap_device"));
  }

  useEffect(() => {
    getSources().catch((err) => console.error(err));

    // no device yet, let's show the config window
    if (currentDevice === "") {
      getCurrentWindow().show();
    }
  }, []);

  async function changeKeyCombo(keyCombo: string) {
    keyCombo = keyCombo.replace(/\s/g, "");
    try {
      // await unregisterAll();
      // await register(keyCombo, (e) => {
      //   console.log(e);
      // });
      await invoke("set_hotkey", { shortcut: keyCombo });
    } catch (err) {
      console.error(
        "error occured while changing key combo to %s",
        keyCombo,
        err,
      );
      return false;
    }

    console.debug("key combo set", keyCombo);

    return true;
  }

  return (
    <main className="container">
      <h1 className="title">FUNNY MIC!</h1>
      <h2>Config</h2>
      <div className="options">
        <div className="row">
          <label htmlFor="micSelect">Swap to</label>
          <select
            name="micSelect"
            id="micSelect"
            onChange={async (e) => {
              await invoke("set_swap_device", {
                swapTo: e.currentTarget.value,
              });
              await getSources();
            }}
            defaultValue={currentDevice}
          >
            <option value="" disabled hidden>
              Select an input device
            </option>
            {Object.entries(sources)
              .sort()
              .map(([name, description]) => {
                return (
                  <option key={name} value={name}>
                    {description}
                  </option>
                );
              })}
          </select>
          <button type="button" onClick={getSources}>
            <RefreshCcw />
          </button>
        </div>
        <div className="row">
          <label htmlFor="micSelect">Swap key combo</label>
          <KeyComboButton onSelect={changeKeyCombo} />
        </div>
      </div>
    </main>
  );
}

export default App;
