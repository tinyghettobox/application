import {AppBar, Box, Button, Container} from "@mui/material";
import {matchPath, Outlet, useLocation} from "react-router-dom";
import {useNavigate} from "react-router";

export default function Root() {
  const navigate = useNavigate();
  const {pathname} = useLocation();

  const handleNavigate = (path: string) => () => navigate(path);

  const activeStyle = (path: string) => (matchPath(path, pathname) ? {background: 'rgba(255,255,255,0.1)'} : {});

  return (
    <>
      <AppBar position={"static"}>
        <Container maxWidth={"xl"}>
          <Box gap={3} sx={{flexGrow: 1, display: {xs: 'none', md: 'flex'}}}>
            <Button onClick={handleNavigate("/systemConfig")} sx={{my: 2, color: 'white', display: 'block', ...activeStyle('/systemConfig')}}>
              System configuration
            </Button>
            <Button onClick={handleNavigate("/spotifyConfig")} sx={{my: 2, color: 'white', display: 'block', ...activeStyle('/spotifyConfig')}}>
              Spotify configuration
            </Button>
            <Button onClick={handleNavigate("/mediaLibrary")} sx={{my: 2, color: 'white', display: 'block', ...activeStyle('/mediaLibrary')}}>
              Media library
            </Button>
          </Box>
        </Container>
      </AppBar>
      <Container maxWidth={"xl"}>
        <Outlet />
      </Container>
    </>
  )
}