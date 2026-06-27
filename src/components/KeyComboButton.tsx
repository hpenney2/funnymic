import { useCallback, useEffect, useRef, useState } from "react";

// /Media|Audio/ are technically allowed!
// https://github.com/tauri-apps/global-hotkey/blob/8c8bcd5b28d6952ab50576e315be828a46aa9f1a/src/hotkey.rs#L233
const SPECIAL_KEYS =
  /Control|Alt|Shift|Super|Channel|Microphone|TV|Unidentified/;

const NO_COMBO = "Click to set";

function fixKeyCode(code: string) {
  return code.replace(/Key|Digit|Arrow/g, "");
}

export default function KeyComboButton({
  onSelect,
  startCombo = NO_COMBO,
}: {
  onSelect?: (keyCombo: string) => Promise<boolean> | boolean;
  startCombo?: string;
}) {
  const ref = useRef<HTMLButtonElement>(null);
  const [settingCombo, setSettingCombo] = useState(false);
  const [combo, setCombo] = useState(startCombo);
  const [proposedCombo, setProposedCombo] = useState("...");
  const [newComboOk, setNewComboOk] = useState(false);

  useEffect(() => {
    setCombo(startCombo);
  }, [startCombo]);

  const keyEventListener = useCallback((event: KeyboardEvent) => {
    event.preventDefault();

    let newCombo = "";
    if (event.ctrlKey) newCombo += "Ctrl + ";
    if (event.altKey) newCombo += "Alt + ";
    if (event.shiftKey) newCombo += "Shift + ";
    if (event.metaKey) newCombo += "Super + ";

    if (!SPECIAL_KEYS.test(event.key)) {
      newCombo += fixKeyCode(event.code);
      // newCombo += event.key.toUpperCase();
      setNewComboOk(true);
    } else {
      newCombo += "...";
      setNewComboOk(false);
    }

    setProposedCombo(newCombo);
  }, []);

  async function pressed() {
    if (!settingCombo) {
      document.addEventListener("keydown", keyEventListener);
    } else {
      document.removeEventListener("keydown", keyEventListener);

      let isOk = newComboOk;
      let newCombo: string;
      if (isOk) {
        if (onSelect) {
          // if onSelect returns false, consider the new combo to fail
          isOk = await onSelect(proposedCombo);

          if (isOk) newCombo = proposedCombo;
          else newCombo = NO_COMBO;
        } else {
          newCombo = proposedCombo;
        }
      } else {
        newCombo = combo;
      }

      setCombo(newCombo);
      setProposedCombo(newCombo); // sync back for visual parity
    }

    setSettingCombo(!settingCombo);
  }

  // remove the event listener when unmounting
  useEffect(() => {
    return () => document.removeEventListener("keydown", keyEventListener);
  }, []);

  return (
    <button
      ref={ref}
      className={`key ${settingCombo ? "key-active" : ""}`}
      type="button"
      onClick={pressed}
    >
      {settingCombo ? proposedCombo : combo}
    </button>
  );
}
