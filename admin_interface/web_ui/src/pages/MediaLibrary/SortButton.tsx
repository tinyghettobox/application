import {
  Button,
  FormControl,
  FormControlLabel,
  FormLabel, Grid, IconButton,
  Popover,
  Radio,
  RadioGroup, Stack,
  TextField,
  Typography
} from "@mui/material";
import {useState, MouseEvent, ChangeEvent} from "react";
import {Add, KeyboardArrowDown, Remove} from "@mui/icons-material";
import {LibraryEntry} from "@db-models/LibraryEntry";
import styles from "./MediaLibrary.module.scss";

interface Props {
  libraryEntries: LibraryEntry[];

  onSorted(libraryEntries: LibraryEntry[]): void;
}

export default function SortButton({libraryEntries, onSorted}: Props) {
  const [open, setOpen] = useState(false);
  const [anchorElement, setAnchorElement] = useState<null | HTMLElement>(null);
  const [patternList, setPatternList] = useState(['^.*?(\\d+).*$']);
  const [direction, setDirection] = useState('asc');

  let regexList: RegExp[] = [];
  let regexError = '';
  try {
    regexList = patternList.map(pattern => new RegExp(pattern, 'i'));
  } catch (e) {
    regexError = `Invalid regex: ${e}`;
  }
  const sortedLibraryEntries = libraryEntries
    .slice(0)
    .sort((a, b) => {
      for (let i = 0; i < regexList.length; i++) {
        const regex = regexList[i];
        let weight = (regexList.length - i) * 10 * (direction === 'asc' ? 1 : -1);
        const matchA = a.name.match(regex);
        const matchB = b.name.match(regex);

        if (!matchA && !matchB) {
          continue;
        }
        if (matchA && !matchB) {
          return weight * -1;
        }
        if (!matchA && matchB) {
          return weight;
        }
        if (Array.isArray(matchA) && Array.isArray(matchB)) {
          if (matchA[1] && !matchB[1]) {
            return weight * -1;
          }
          if (!matchA[1] && matchB[1]) {
            return weight;
          }
          if (matchA[1] && matchB[1]) {
            const valueA = Number(matchA[1].replace(',', '.')); // Ensure english number format
            const valueB = Number(matchB[1].replace(',', '.')); // Ensure english number format
            if (!Number.isNaN(valueA) && !Number.isNaN(valueB)) {
              return valueA > valueB ? weight : weight * -1;
            }
            if (Number.isNaN(valueA) && !Number.isNaN(valueB)) {
              return weight * -1;
            }
            if (!Number.isNaN(valueA) && Number.isNaN(valueB)) {
              return weight;
            }
            return matchA[1] > matchB[1] ? weight : weight * -1;
          }
        }
        console.warn(`Regex ${regex} matched but no captured group found. This should not happen. A: ${a.name}, B: ${b.name}`);
      }
      return a.name < b.name ? 1 : -1;
    })
    .map((entry, index) => {
      entry.sortKey = index;
      return entry;
    });

  const handleClick = (event: MouseEvent<HTMLButtonElement>) => {
    setOpen(true);
    setAnchorElement(event.currentTarget);
  };

  const handleClose = () => {
    setOpen(false);
    setAnchorElement(null);
  };

  const submit = () => {
    handleClose();
    onSorted(sortedLibraryEntries);
  };

  const handlePatternChange = (index: number) => (event: ChangeEvent<HTMLInputElement>) => {
    setPatternList(list => {
      list[index] = event.target.value;
      return [...list];
    });
  };

  const handleAddPattern = () => {
    setPatternList(list => [...list, '^.*?(\\d+).*$']);
  }

  const handleRemovePattern = (index: number) => () => {
    setPatternList(list => list.filter((_, i) => i !== index));
  }

  const handleDirectionChange = (event: ChangeEvent<HTMLInputElement>) => {
    setDirection(event.target.value);
  };

  return (
    <>
      <Button onClick={handleClick}>
        Sort items
        <KeyboardArrowDown/>
      </Button>
      <Popover
        open={open}
        anchorEl={anchorElement}
        onClose={handleClose}
        anchorOrigin={{
          vertical: 'bottom',
          horizontal: 'left',
        }}
      >
        <div className={styles.sortPopover}>
          <Stack spacing={1} className={styles.patternList}>
            {patternList.map((pattern, index) =>
              <Grid container key={index}>
                <Grid item flex={"1 1"}>
                  <TextField
                    label={"Extractor pattern"}
                    size={"small"}
                    fullWidth
                    value={pattern}
                    onChange={handlePatternChange(index)}
                    InputProps={{
                      startAdornment: '/',
                      endAdornment: '/'
                    }}
                  />
                </Grid>
                <Grid item flex={"0 0"}>
                  {index === patternList.length - 1 ? (
                    <IconButton size={"small"} onClick={handleAddPattern}><Add/></IconButton>
                  ) : (
                    <IconButton size={"small"} onClick={handleRemovePattern(index)}><Remove/></IconButton>
                  )}
                </Grid>
              </Grid>
            )}
            {regexError && <Typography color={"error"}>{regexError}</Typography>}
          </Stack>
          <FormControl>
            <FormLabel id="direction-label">Direction</FormLabel>
            <RadioGroup
              row
              aria-labelledby="direction-label"
              value={direction}
              onChange={handleDirectionChange}
            >
              <FormControlLabel value="asc" control={<Radio/>} label="Ascending"/>
              <FormControlLabel value="desc" control={<Radio/>} label="Descending"/>
            </RadioGroup>
          </FormControl>
          <div>
            <Button onClick={submit} variant={"outlined"} size={"small"}>Apply sorting</Button>
          </div>
          <div className={styles.exampleList}>
            <Typography variant={"subtitle1"}>Example order</Typography>
            <div style={{overflow: "auto", maxHeight: 300}}>
              {sortedLibraryEntries.slice(0, 100).map((entry, index) => (
                <div key={entry.id} className={styles.exampleItem}>{index + 1}. {entry.name}</div>
              ))}
            </div>
          </div>
        </div>
      </Popover>
    </>
  )
}