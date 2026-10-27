import { desktopApp } from '../lib/app-config';

// Application-owned component: replace branding/content here. The OS supplies
// native caption buttons, accessibility, resize/snap and keyboard behavior.
export function TitleBar() {
  return <header className="desktop-titlebar" aria-label="Application title bar" style={{ height: desktopApp.titleBar.height }}>
    <div className="desktop-titlebar-content">
      <svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="none"><rect x="3" y="3" width="18" height="18" rx="5" stroke="currentColor" strokeWidth="1.5" /><path d="m7 12 3 3 7-7" stroke="currentColor" strokeWidth="1.5" /></svg>
      <span>{desktopApp.title}</span>
    </div>
  </header>;
}
