import {createBrowserRouter, redirect, RouterProvider} from 'react-router-dom';
import SpotifyConfig from '@/pages/SpotifyConfig/SpotifyConfig';
import SystemConfig from "@/pages/SystemConfig/SystemConfig";
import MediaLibrary from "@/pages/MediaLibrary/MediaLibrary";
import Setup from "@/pages/Setup/Setup";
import Notification from "@/components/Notification";
import Root from "@/pages/Root";
import './App.model.css'
import {CssBaseline, ThemeProvider} from "@mui/material";
import {theme} from "@/theme";

async function setupGuard() {
  try {
    const res = await fetch('/api/system/config');
    if (res.ok) {
      const data = await res.json();
      if (!data.setup_complete) {
        return redirect('/setup/0');
      }
    }
  } catch {
    // If the config can't be fetched, let the user through.
  }
  return null;
}

const router = createBrowserRouter([
  // Setup wizard — outside Root layout, no nav bar.
  { path: '/setup/:step?', element: <Setup /> },
  {
    path: '/',
    element: <Root />,
    loader: setupGuard,
    children: [
      {path: '', loader: () => redirect('/systemConfig')},
      {path: 'systemConfig', element: <SystemConfig />, id: 'System configuration'},
      {path: 'spotifyConfig/:step?', element: <SpotifyConfig />, id: 'Spotify configuration'},
      {path: 'mediaLibrary/:id?', element: <MediaLibrary />, id: 'Media library'}
    ]
  }
]);

export const App = () => {

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <RouterProvider router={router} />
      <Notification />
    </ThemeProvider>
  );
}