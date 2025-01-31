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

  const regexList = patternList.map(pattern => new RegExp(pattern, 'i'));
  const sortedLibraryEntries = libraryEntries
    .slice(0)
    .sort((a, b) =>
      regexList
        .map((regex, i) => {
          const weight = regexList.length - i;
          const matchA = a.name.match(regex);
          const matchB = b.name.match(regex);
          let valueA: string | number = (matchA ? (matchA[1] || matchA[0]) : a.name).replace(',', '.'); // Ensure english number format
          let valueB: string | number = (matchB ? (matchB[1] || matchB[0]) : b.name).replace(',', '.'); // Ensure english number format

          if (!Number.isNaN(Number(valueA))) {
            valueA = Number(valueA);
          }
          if (!Number.isNaN(Number(valueB))) {
            valueB = Number(valueB);
          }

          if (direction === 'desc') {
            return valueA > valueB ? weight * -1 : weight;
          }
          return valueA > valueB ? weight : weight * -1;
        })
        .reduce((sum, weight) => sum + weight, 0)
    )
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
            <div>
              {sortedLibraryEntries.slice(0, 5).map((entry, index) => (
                <div key={entry.id} className={styles.exampleItem}>{index + 1}. {entry.name}</div>
              ))}
            </div>
          </div>
        </div>
      </Popover>
    </>
  )
}