import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";
import { RefreshCcw } from "lucide-react";
import KeyComboButton from "./components/KeyComboButton";

// const asleep = (ms: number) =>
//   new Promise((resolve) => setTimeout(resolve, ms));

function App() {
  const [sources, setSources] = useState<Record<string, string>>({});
  const [currentDevice, setCurrent] = useState<string>("");

  const [currentCombo, setCurrentCombo] = useState<string | undefined>(
    undefined,
  );

  async function getSources() {
    setSources(await invoke("get_device_list"));

    let current = await invoke<string>("get_swap_device");
    setCurrent(current);

    return current;
  }

  useEffect(() => {
    getSources()
      .then((current) => {
        // no device yet, let's show the config window
        if (current === "") {
          getCurrentWindow().show();
        }
        return invoke<string | undefined>("get_hotkey");
      })
      .then((hotkey) => {
        setCurrentCombo(hotkey);
      })
      .catch((err) => console.error(err));
  }, []);

  async function changeKeyCombo(keyCombo: string) {
    // keyCombo = keyCombo.replace(/\s/g, "");
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
            value={currentDevice}
          >
            <option value="" disabled hidden>
              Select an input device
            </option>
            {Object.entries(sources)
              .sort((a, b) => a[1].localeCompare(b[1])) // sort by value, not keys
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
          <KeyComboButton onSelect={changeKeyCombo} startCombo={currentCombo} />
        </div>
      </div>
    </main>
  );
}

export default App;
