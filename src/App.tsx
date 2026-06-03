import { useEffect } from 'react';
import { MainLayout } from './components/layout/MainLayout';
import { takeScreenshot } from './utils/tauri';

function App() {
  // Dev screenshot: Cmd+Shift+S (macOS) / Ctrl+Shift+S (others)
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === 'S') {
        e.preventDefault();
        takeScreenshot()
          .then((path) => console.log('[Screenshot] Saved to:', path))
          .catch((err) => console.error('[Screenshot] Failed:', err));
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  return <MainLayout />;
}

export default App;
