import { Toaster } from "sonner";
import "./index.css";

import { Sidebar } from "./components/layout/Sidebar";
import { DeviceTopBar } from "./components/layout/DeviceTopBar";
import { MainArea } from "./components/layout/MainArea";
import { TitleBar } from "./components/layout/TitleBar";

import { DevicesView } from "./views/DevicesView";
import { AppsView } from "./views/AppsView";
import { LogcatView } from "./views/LogcatView";
import { FilesView } from "./views/FilesView";
import { SettingsView } from "./views/SettingsView";

import { useAppStore } from "./store/useAppStore";
import { useSettings } from "./hooks/useSettings";
import { useDevices } from "./hooks/useDevices";
import { SetupModal } from "./components/layout/SetupModal";

function App() {
  useSettings();
  useDevices();
  const activeView = useAppStore((s) => s.activeView);

  const viewMap = {
    devices: <DevicesView />,
    apps: <AppsView />,
    logcat: <LogcatView />,
    files: <FilesView />,
    settings: <SettingsView />,
  } as const;

  return (
    <>
      <TitleBar />
      <div
        style={{
          display: "flex",
          height: "calc(100vh - 32px)",
          width: "100vw",
          overflow: "hidden",
          background: "var(--color-surface)",
          color: "var(--color-text-primary)",
          fontFamily: "var(--font-sans)",
        }}
      >
        {/* ── Left: icon-only sidebar (56px) ── */}
        <Sidebar />

        {/* ── Right: sticky topbar + scrollable main content ── */}
        <div style={{ display: "flex", flexDirection: "column", flex: 1, minWidth: 0 }}>
          <DeviceTopBar />
          <MainArea>{viewMap[activeView]}</MainArea>
        </div>
      </div>

      {/* Global toast notifications */}
      <Toaster
        position="bottom-right"
        theme="dark"
        toastOptions={{
          style: {
            background: "var(--color-surface-card)",
            border: "1px solid var(--color-surface-border)",
            color: "var(--color-text-primary)",
            fontFamily: "var(--font-sans)",
            fontSize: "13px",
          },
        }}
      />
      <SetupModal />
    </>
  );
}

export default App;
