import { getCurrentWindow } from "@tauri-apps/api/window";
import { X, Minus, Square } from "lucide-react";
import logo from "../../assets/logo.png";

const appWindow = getCurrentWindow();

export function TitleBar() {
  const isMac = typeof window !== 'undefined' && /Mac|iPhone|iPod|iPad/i.test(navigator.userAgent);

  return (
    <div 
      data-tauri-drag-region
      className={`titlebar ${isMac ? 'is-mac' : ''}`}
    >
      {isMac ? (
        <>
          <div className="titlebar-controls left">
            <button className="titlebar-btn mac-dot close" onClick={() => appWindow.close()} title="Close" />
            <button className="titlebar-btn mac-dot minimize" onClick={() => appWindow.minimize()} title="Minimize" />
            <button className="titlebar-btn mac-dot maximize" onClick={() => appWindow.toggleMaximize()} title="Maximize" />
          </div>
          
          <div className="titlebar-title centered">
            <img src={logo} className="titlebar-logo" alt="" />
            <span>Open Quest Hub</span>
          </div>
          
          {/* Spacer to keep title centered */}
          <div style={{ width: 52 }} />
        </>
      ) : (
        <>
          <div className="titlebar-title">
            <img src={logo} className="titlebar-logo" alt="" />
            <span>Open Quest Hub</span>
          </div>

          <div className="titlebar-controls right">
            <button className="titlebar-btn" onClick={() => appWindow.minimize()} title="Minimize">
              <Minus size={14} />
            </button>
            <button className="titlebar-btn" onClick={() => appWindow.toggleMaximize()} title="Maximize">
              <Square size={10} />
            </button>
            <button className="titlebar-btn hover-danger" onClick={() => appWindow.close()} title="Close">
              <X size={14} />
            </button>
          </div>
        </>
      )}
    </div>
  );
}
