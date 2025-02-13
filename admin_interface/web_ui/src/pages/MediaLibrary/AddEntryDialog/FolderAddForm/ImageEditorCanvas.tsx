import React, {useEffect, useRef, useState} from "react";
import {LibraryEntry} from "@db-models/LibraryEntry";
import {Slider} from "@mui/material";
import {arrayToBase64} from "@/util/base64";

interface Props {
  folder?: LibraryEntry;
  onResize: (image: Array<number>) => void
}

export default function ImageEditor({folder}: Props) {
  const [state, setState] = useState({scale: 1, translate: {x: 0, y: 0}});
  const editStart = useRef({x: 0, y: 0});
  const canvas = useRef(document.createElement('canvas'));
  const image = useRef<HTMLImageElement | undefined>();

  useEffect(() => {
    (async () => {
      if (folder?.image) {
        const img = new Image();
        const onLoad = () => {
          img.removeEventListener('load', onLoad);
          image.current = img;
        };
        img.addEventListener('load', onLoad);
        img.src = `data:image/png;base64,${arrayToBase64(folder?.image)}`;
      }
    })();
  }, [folder?.image]);

  const handleEditStart = (event: React.MouseEvent<Element>) => {
    editStart.current.x = event.pageX;
    editStart.current.y = event.pageY;
    window.addEventListener('mousemove', handleEdit);
    window.addEventListener('mouseup', handleEditStop);
  }

  const handleEdit = (event: MouseEvent) => {
    let diffX = editStart.current.x === 0 ? 0 : event.pageX - editStart.current.x;
    let diffY = editStart.current.y === 0 ? 0 : event.pageY - editStart.current.y;

    if (image.current) {
      canvas.current.getContext('2d')?.drawImage(image.current, state.translate.x + diffX, state.translate.y + diffY)
    }
  }

  const handleEditStop = (event: MouseEvent) => {
    window.removeEventListener('mousemove', handleEdit);
    window.removeEventListener('mouseup', handleEditStop);

    let diffX = editStart.current.x === 0 ? 0 : event.pageX - editStart.current.x;
    let diffY = editStart.current.y === 0 ? 0 : event.pageY - editStart.current.y;

    setState(old => ({ ...old, translate: {x: state.translate.x + diffX, y: state.translate.y + diffY} }));
  }
  
  const handleScaleChange = (_event: Event, value: number | number[]) => {
    setState(old => ({ ...old, scale: Array.isArray(value) ? value[0] : value }));
  }

  return (
    <div>
      <div style={{position: 'relative', overflow: 'hidden'}}>
        <canvas ref={canvas} style={{width: '128px', height: '128px', borderRadius: '64px'}} />

        <div data-edit-type="scale" style={{
          position: 'absolute',
          top: '0',
          left: '0',
          width: '128px',
          height: '128px',
        }}>
          <svg viewBox="0 0 128 128" width="128" style={{display: 'block', cursor: 'default'}}>
            <defs>
              <mask id="mask">
                <rect x="0" y="0" width="128" height="128" fill="#ffffff"/>
                <circle cx="64" cy="64" r="64"/>
              </mask>
            </defs>
            <rect x="0" y="0" width="128" height="128" mask="url(#mask)" fillOpacity="0.5"></rect>
            <circle
              cx="64"
              cy="64"
              r="64"
              fill="transparent"
              data-edit-type="move"
              style={{cursor: 'move'}}
              onMouseDown={handleEditStart}
            />
          </svg>
        </div>
      </div>
      <div style={{width: '100%'}}>
        <Slider
          size="small"
          defaultValue={1}
          min={0.01}
          max={3}
          step={0.01}
          aria-label="Small"
          valueLabelDisplay="auto"
          onChange={handleScaleChange}
        />
      </div>
    </div>
  )
}