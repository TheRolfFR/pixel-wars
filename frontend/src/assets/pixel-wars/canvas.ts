import type CanvasElementController from './CanvasController';
import type { CanvasPixels, Color } from './CanvasController';
import { CanvasPaletteStore } from './stores';

let ColorPaletteLocal: Color[] = [];

export function encodeColor(input: Color): number {
  const input_str = input.toString();
  return ColorPaletteLocal.findIndex(c => c.toString() === input_str);
}

export function decodeColor(color: number): Color {
  const result = ColorPaletteLocal[color];
  return result;
}

interface CanvasInfoSize {
  width: number,
  height: number,
  chunkSize: number,
}

export async function initialLoad(canvasController: CanvasElementController) {
  const canvasJSON = await fetch(window.location.protocol+"//"+window.location.host+'/api/canvas')
      .then(t => t.json())
      .catch(e => {
        console.error(e);
        return Promise.reject(e);
      });

  //* Size
  const canvasSize = canvasJSON['size'] as CanvasInfoSize;

  //* Colors
  ColorPaletteLocal = canvasJSON['colors'].map((e: [number, number, number]) => [...e, 255]);
  CanvasPaletteStore.set(ColorPaletteLocal);
  CanvasPaletteStore.subscribe(palette => {
    ColorPaletteLocal = palette;
  });

  //* Pixels
  const canvas_obj_bytes: Record<string, Array<number>> = {};
  for (const chunkRowIndex in canvasJSON['canvas']) {
    const chunkRow = canvasJSON['canvas'][chunkRowIndex];
    for (const chunkColIndex in chunkRow) {
      const chunk = chunkRow[chunkColIndex];
      const canvasString = atob(chunk);
      const bytes = new Uint8Array(canvasString.length);
      canvas_obj_bytes[`${chunkRowIndex},${chunkColIndex}`] = new Array(bytes.length);
      for(let i = 0; i < canvasString.length; i++){
        canvas_obj_bytes[`${chunkRowIndex},${chunkColIndex}`][i] = canvasString.charCodeAt(i)
      }
    }
  }

  canvasController.setSize(canvasSize.width, canvasSize.height);
  const imageData = canvasStringToColorList(canvas_obj_bytes, canvasSize);
  canvasController.putCanvasPixels(imageData);
}

function canvasStringToColorList(
  canvasArray: Record<string, Array<number>>,
  canvasSize: CanvasInfoSize,
): CanvasPixels {
  const canvasColors: Array<Color> = new Array(canvasSize.height * canvasSize.width).fill(null);
  for (const [key, chunk_bytes] of Object.entries(canvasArray))
  {
    // those are reversed somehow
    const [chunk_index_x, chunk_index_y] = key.split(',').map(s => Number.parseInt(s, 10));
    let chunk_pos_x = 0;
    let chunk_pos_y = 0;
    for (let array_i  = 0; array_i < chunk_bytes.length; array_i++)
    {
      let canvas_pos_y = chunk_index_y * canvasSize.chunkSize + chunk_pos_y;
      let canvas_pos_x = chunk_index_x * canvasSize.chunkSize + chunk_pos_x;

      if(canvas_pos_x < canvasSize.width && canvas_pos_y < canvasSize.height)
      {
        const color_code = chunk_bytes[array_i] >> 4;
        const color = decodeColor(color_code);
        canvasColors[canvas_pos_y * canvasSize.width + canvas_pos_x] = color;
      }

      chunk_pos_x += 1;
      if(chunk_pos_x >= canvasSize.chunkSize) {
        chunk_pos_x = 0;
        chunk_pos_y += 1;
      }

      canvas_pos_y = chunk_index_y * canvasSize.chunkSize + chunk_pos_y;
      canvas_pos_x = chunk_index_x * canvasSize.chunkSize + chunk_pos_x;

      if(canvas_pos_x < canvasSize.width && canvas_pos_y < canvasSize.height)
      {
        const color_code = chunk_bytes[array_i] & 15;
        const color = decodeColor(color_code);
        canvasColors[canvas_pos_y * canvasSize.width + canvas_pos_x] = color;
      }

      chunk_pos_x += 1;
      if(chunk_pos_x >= canvasSize.chunkSize) {
        chunk_pos_x = 0;
        chunk_pos_y += 1;
      }
    }
  }

  return {
    colors: canvasColors,
    width:canvasSize.width,
    height:canvasSize.height
  };
}
