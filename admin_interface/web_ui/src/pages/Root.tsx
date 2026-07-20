import {AppBar, Box, Button, Container, Drawer, IconButton, List, ListItem, ListItemButton, ListItemText, Toolbar} from "@mui/material";
import {matchPath, Outlet, useLocation} from "react-router-dom";
import {useNavigate} from "react-router";
import {Menu as MenuIcon} from "@mui/icons-material";
import {useState} from "react";

const NAV_ITEMS = [
  {label: 'System configuration', path: '/systemConfig'},
  {label: 'Spotify configuration', path: '/spotifyConfig'},
  {label: 'Media library', path: '/mediaLibrary'},
];

export default function Root() {
  const navigate = useNavigate();
  const {pathname} = useLocation();
  const [drawerOpen, setDrawerOpen] = useState(false);

  const handleNavigate = (path: string) => () => {
    navigate(path);
    setDrawerOpen(false);
  };

  const activeStyle = (path: string) => (matchPath(path, pathname) ? {background: 'rgba(255,255,255,0.1)'} : {});

  return (
    <>
      <AppBar position={"static"}>
        <Container maxWidth={"xl"}>
          <Toolbar disableGutters>
            {/* Desktop nav */}
            <Box gap={3} sx={{flexGrow: 1, display: {xs: 'none', md: 'flex'}}}>
              {NAV_ITEMS.map(item => (
                <Button key={item.path} onClick={handleNavigate(item.path)} sx={{my: 2, color: 'white', display: 'block', ...activeStyle(item.path)}}>
                  {item.label}
                </Button>
              ))}
            </Box>
            {/* Mobile hamburger */}
            <Box sx={{display: {xs: 'flex', md: 'none'}}}>
              <IconButton color="inherit" onClick={() => setDrawerOpen(true)} size="large" aria-label="menu">
                <MenuIcon />
              </IconButton>
            </Box>
          </Toolbar>
        </Container>
      </AppBar>

      {/* Mobile drawer */}
      <Drawer anchor="left" open={drawerOpen} onClose={() => setDrawerOpen(false)}>
        <List sx={{width: 260}}>
          {NAV_ITEMS.map(item => (
            <ListItem key={item.path} disablePadding>
              <ListItemButton selected={!!matchPath(item.path, pathname)} onClick={handleNavigate(item.path)}>
                <ListItemText primary={item.label} />
              </ListItemButton>
            </ListItem>
          ))}
        </List>
      </Drawer>

      <Container maxWidth={"xl"}>
        <Outlet />
      </Container>
    </>
  )
}