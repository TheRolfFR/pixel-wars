import type CanvasElementController from './CanvasController';
import { decodeColor } from './canvas';
import { ColorPickerStore, OnlineCountStore, TimeoutStore } from './stores';
import { get } from 'svelte/store';

export default class SubscriptionController {
  websocketServer: WebSocket;
  canvasController: CanvasElementController;

  constructor(canvasController: CanvasElementController) {
    this.canvasController = canvasController;
  }


  public async initConnection() {
    const cookies = await fetch("http://"+window.location.host+'/pixelwars/api/getSession');
    if (!(cookies.status == 401 || cookies.status == 200)) {
      //TODO: show that something went wrong while trying to use session
      return;
    }
    window.dispatchEvent(new CustomEvent<{ done: boolean }>("sessionLoaded", { detail: { done: true } }));
    this.websocketServer = new WebSocket('ws://' + window.location.host + '/pixelwars/api/subscribe');
    this.websocketServer.addEventListener("message", this.receiveMessageHandler());

    window.addEventListener("pixelClicked", async (ev: CustomEvent) => {
      const coords = ev.detail as { x: number, y: number };
      let timeout = get(TimeoutStore);
      if (timeout.remainingPixels == 0) return 0;
      timeout.remainingPixels--;
      TimeoutStore.set(timeout);
      const color = get(ColorPickerStore);
      await this.sendUpdate(coords.x, coords.y, color);
      this.canvasController.putPixelCanvas(coords.x, coords.y, decodeColor(color));
    })
  }

  public async sendUpdate(x: number, y: number, color: number) {
    if (color >= 16) throw new Error(`illegal color ${color} must be less than 16...`);
    this.websocketServer.send(this.encodeMessage(x, y, color));
  }

  private encodeMessage(x: number, y: number, color: number) {
    const buffer = new ArrayBuffer(5);
    const dataView = new DataView(buffer);

    dataView.setUint16(0, x, false);
    dataView.setUint16(2, y, false);
    dataView.setUint8(4, color);

    return buffer;
  }

  private decodeMessage(buffer: ArrayBuffer) {
    const dataView = new DataView(buffer);

    const x = dataView.getUint16(0, false);
    const y = dataView.getUint16(2, false);
    const color = dataView.getUint8(4);

    return { x, y, color };
  }

  private receiveMessageHandler() {
    const subscription: SubscriptionController = this;
    return async (message: MessageEvent<Blob|string>) => {
      if(typeof(message.data) === 'string')
      {
        const [command, args] = message.data.split(' ');
        if(command === '/count')
        {
          const count = Number.parseInt(args[0], 10);
          OnlineCountStore.set(count)
        }
        return;
      }

      const { x, y, color } = subscription.decodeMessage(await message.data.arrayBuffer());

      subscription.canvasController.putPixelCanvas(x, y, decodeColor(color));
    };
  }

}
