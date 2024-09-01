import type CanvasElementController from './CanvasController';
import type { CanvasPixels, Color } from './CanvasController';
import { CanvasPaletteStore } from './stores';

let ColorPaletteLocal: Color[] = [];

export function encodeColor(input: Color): number {
  const input_str = input.toString();
  return ColorPaletteLocal.findIndex(c => c.toString() === input_str);
}

export function decodeColor(color: number): Color {
  return ColorPaletteLocal[color];
}

export async function initialLoad(canvasController: CanvasElementController) {
  let canvasResponse: Response;
  try {
    canvasResponse = await fetch(window.location.protocol+"//"+window.location.host+'/api/canvas');
  } catch (err) {
    console.log(err);
  }

  const canvasJSON = await canvasResponse.json();
  const canvasString = atob(canvasJSON['canvas']);
  const bytes = new Uint8Array(canvasString.length);
  for(let i = 0; i < canvasString.length; i++){
    bytes[i] = canvasString.charCodeAt(i)
  }
  const canvasSize = canvasJSON['size'] as { width: number; height: number };

  ColorPaletteLocal = canvasJSON['colors'].map(e => [...e, 255]);
  CanvasPaletteStore.set(ColorPaletteLocal);
  CanvasPaletteStore.subscribe(palette => {
    ColorPaletteLocal = palette;
  });

  const imageData = canvasStringToColorList(bytes, canvasSize);
  canvasController.putCanvasPixels(imageData);
}

function canvasStringToColorList(
  canvasArray: Uint8Array,
  canvasSize: { width: number; height: number }
): CanvasPixels {
  const canvasPixels = [];
  for (let i = 0; i < canvasArray.length; i++) {
    canvasPixels.push(canvasArray[i] >> 4);
    canvasPixels.push(canvasArray[i] & 15);
  }

  const colors: Color[] = [];
  for (let i = 0; i < canvasPixels.length; i++) {
    if(canvasPixels[i] > 15) console.log(canvasPixels[i]);
    const color: Color = decodeColor(canvasPixels[i]);
    colors.push(color);
  }

  return {
    colors,
    width:canvasSize.width,
    height:canvasSize.height
  };
}
