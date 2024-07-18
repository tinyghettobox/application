import {createTheme} from "@mui/material";

export const theme = createTheme({
  typography: {
    fontFamily: 'Poppins, Roboto, Arial, sans-serif'
  },
  components: {
    MuiSlider: { defaultProps: {size: "small"}},
    MuiSelect: { defaultProps: {size: "small"}},
    MuiTextField: { defaultProps: {size: "small"}},
    MuiOutlinedInput: { defaultProps: {size: "small"}},
    MuiFormControl: {defaultProps: {size: "small"}}
  }
})