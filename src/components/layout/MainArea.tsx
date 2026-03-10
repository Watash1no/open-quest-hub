import type { ReactNode } from "react";

interface MainAreaProps {
  children: ReactNode;
}

/**
 * Scrollable content area rendered below the sticky DeviceTopBar.
 * Fills remaining vertical space and handles its own scroll.
 */
export function MainArea({ children }: MainAreaProps) {
  return (
    <main
      className="animate-fade-in"
      style={{ flex: 1, overflowY: "auto", minHeight: 0 }}
    >
      {children}
    </main>
  );
}
