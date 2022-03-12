import { useState } from "react";
import logo from "./logo.svg";
import "./App.css";

const Keyboard = () => {
  const mainSize = [15, 5]
  const keyMatrixs = (
    (
      "Esc #1234567890-= Backspace \n" +
      "Tab #QWERTYUIOP[]\\ \n" +
      "CapsLock #ASDFGHJKL;' Enter\n" +
      "LShift #ZXCVBNM,./ RShift\n" +
      "LCtrl LWin LAlt Space RAlt Menu RCtrl Fn\n"
    )
      .trim().split(/\n/)
      .map(row => row.trim().split(/\s+/))
      .map(row => row.flatMap((keyGroup) => keyGroup[0].match(/#/) ? keyGroup.slice(1).split('') : [keyGroup]))
  )
  const bott = 3.7 / 3
  const keyWidths = {
    Backspace: mainSize[0] - 13,
    Tab: 1.5, '\\': mainSize[0] - 1.5 - 12,
    CapsLock: 1.8, Enter: mainSize[0] - 1.8 - 11,
    LShift: 2.3, RShift: mainSize[0] - 2.3 - 10,
    LCtrl: bott, LWin: bott, LAlt: bott,
    Space: mainSize[0] - 7 * bott,
    RCtrl: bott, RWin: bott, RAlt: bott, Menu: bott,
    Fn: bott
  } as any
  const kbdParentStyle = {
    width: mainSize[0] * 4 + 'rem',
    height: mainSize[1] * 4 + 'rem'
  };
  const kbdStyle = (keyName: string) => ({
    width: (keyWidths[keyName] ?? 1) * 4 + 'rem',
    height: 1 * 4 + 'rem'
  });
  return <div style={
    kbdParentStyle
  }>
    {keyMatrixs.map(keyRow => <>{keyRow.map(e => <kbd style={kbdStyle(e)}>{e}</kbd>)}</>)}
  </div>
}
function App() {
  const [count, setCount] = useState(0);

  return (
    <div className="App">3
      <header className="App-header">
        <Keyboard />
      </header>
    </div>
  );
}

export default App;
