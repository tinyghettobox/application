import {createBrowserRouter, redirect, RouterProvider} from 'react-router-dom';
import SpotifyConfig from '@/pages/SpotifyConfig/SpotifyConfig';
import SystemConfig from "@/pages/SystemConfig/SystemConfig";
import MediaLibrary from "@/pages/MediaLibrary/MediaLibrary";
import Notification from "@/components/Notification";
import Root from "@/pages/Root";
import './App.model.css'
import {CssBaseline, ThemeProvider} from "@mui/material";
import {theme} from "@/theme";

export const App = () => {
  const router = createBrowserRouter([
    {path: '/', element: <Root />, children: [
      {path: '', loader: () => redirect('/systemConfig')},
      {path: 'systemConfig', element: <SystemConfig />, id: 'System configuration'},
      {path: 'spotifyConfig/:step?', element: <SpotifyConfig />, id: 'Spotify configuration'},
      {path: 'mediaLibrary/:id?', element: <MediaLibrary />, id: 'Media library'}
    ]}
  ]);

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <RouterProvider router={router} />
      <Notification />
    </ThemeProvider>
  );
}