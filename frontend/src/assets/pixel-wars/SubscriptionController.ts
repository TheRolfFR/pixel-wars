import type CanvasElementController from './CanvasController';
import { decodeColor, encodeColor } from './canvas';
import { ColorPickerStore, OnlineCountStore, TimeoutStore } from './stores';
import { get } from 'svelte/store';
import timeFormat from './utils/timeFormat';

export default class SubscriptionController {
  websocketServer: WebSocket;
  websocketServerCreated: number;
  canvasController: CanvasElementController;

  constructor(canvasController: CanvasElementController) {
    this.canvasController = canvasController;
  }

  public async createWsConnection() {
    const protocol = window.location.protocol.startsWith("https") ? "wss://" : "ws://";
    this.websocketServer = new WebSocket(protocol + window.location.host + '/api/subscribe');
    this.websocketServerCreated = Date.now();
    this.websocketServer.addEventListener("message", this.receiveMessageHandler());
    this.websocketServer.addEventListener("error", (event) => {
      console.error("WebSocket error: ", event);
    })
    this.websocketServer.addEventListener("close", (event) => {
      const code = event.code;
      const duration = timeFormat(Math.round((Date.now() - this.websocketServerCreated) / 1000));
      if(code === 1000) {
        console.info(`WebSocket closed after ${duration} with error code ${code}: Normal Closure`)
      }
      if(code === 1001) {
        console.info(`WebSocket closed after ${duration} with error code ${code}: Going away`)
      } else {
        console.error(`WebSocket closed after ${duration} with error code ${code}`, event);
        console.error(`Websocket closed after ${duration} with following reason: `, event.reason);

        console.log("Reopening socket...");
        this.createWsConnection();
      }
    })
  }

  public async initConnection() {
    const cookies = await fetch(window.location.protocol+"//"+window.location.host+'/api/getSession');
    if (!(cookies.status == 401 || cookies.status == 200)) {
      //TODO: show that something went wrong while trying to use session
      return;
    }
    window.dispatchEvent(new CustomEvent<{ done: boolean }>("sessionLoaded", { detail: { done: true } }));
    await this.createWsConnection();

    window.addEventListener("pixelClicked", async (ev: CustomEvent) => {
      const coords = ev.detail as { x: number, y: number };

      // do not place if same pixel color
      const color = get(ColorPickerStore);
      const { x, y } = coords;
      if(encodeColor(this.canvasController.getPixelCanvas(x, y)) === color) return;

      const timeout = get(TimeoutStore);
      if (timeout.remainingPixels == 0) return 0;
      timeout.remainingPixels--;
      TimeoutStore.set(timeout);
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
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    const subscription: SubscriptionController = this;
    return async (message: MessageEvent<Blob|string>) => {
      if(typeof(message.data) === 'string')
      {
        const [command, ...args] = message.data.split(' ');
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
