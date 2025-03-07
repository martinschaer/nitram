import { AuthenticateAPI } from "nitram/API";

import { NitramResponse } from "nitram/NitramResponse";
import { NitramServerMessage } from "nitram/NitramServerMessage";
import { NitramRequest } from "nitram/NitramRequest";
import { JsonValue } from "nitram/serde_json/JsonValue";

type EventHandler = (data: any) => void;
type ServerMessageHandler = (data: any) => void;
type QueueItem = NitramRequest & {
  resolve: (val: any) => any;
  reject: () => any;
};

function wsStateToString(state: number) {
  switch (state) {
    case WebSocket.CONNECTING:
      return "CONNECTING";
    case WebSocket.OPEN:
      return "OPEN";
    case WebSocket.CLOSING:
      return "CLOSING";
    case WebSocket.CLOSED:
      return "CLOSED";
    default:
      return "UNKNOWN";
  }
}

// =============================================================================
// Server
// =============================================================================
export class Server {
  // -- Public
  is_authenticated = false;

  // -- Private
  private _stop = false;
  private lastState: number = WebSocket.CLOSED;
  private ws: WebSocket;
  private handlers: Map<string, (data: any) => void> = new Map();
  private errorHandlers: Map<string, (data: any) => void> = new Map();
  private eventHandlers: Map<string, EventHandler[]> = new Map();
  private serverMessageHandlers: Map<string, ServerMessageHandler[]> =
    new Map();
  private queue: QueueItem[] = [];

  // -- Constructor
  constructor() {
    this.ws = new WebSocket(`${import.meta.env.VITE_WS_SERVER}/ws`);
    this.check_connection();
    this.init();
  }

  // ---------------------------------------------------------------------------
  // Private Methods
  //

  private process_message_from_server(data: JsonValue) {
    if (data === null) {
      // - null
    } else if (data.hasOwnProperty("key")) {
      // - server messages
      const serverMessageData = data as NitramServerMessage;

      // -- find registered server message handlers
      const handlers = this.serverMessageHandlers.get(serverMessageData.key);
      if (handlers) {
        console.log(`<-- server msg: ${serverMessageData.key}`);
        for (const handler of handlers) {
          handler(serverMessageData.payload);
        }
      } else {
        // -- unhandled server message
        console.log("<-- server msg unhandled: ", serverMessageData.key);
      }
    } else {
      // - message responses
      if (
        data.hasOwnProperty("method") &&
        data.hasOwnProperty("ok") &&
        data.hasOwnProperty("response")
      ) {
        const messageData = data as unknown as NitramResponse;
        if (messageData.ok) {
          let handler = this.handlers.get(messageData.method);
          if (handler) handler(messageData.response);
          else console.warn("!!! Unhandled message", messageData);
        } else {
          let handler = this.errorHandlers.get(messageData.method);
          if (handler) handler(messageData.response);
          else console.warn("!!! Unhandled error", messageData);
        }
      }
    }
  }

  private init() {
    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        if (Array.isArray(data)) {
          for (const item of data) {
            this.process_message_from_server(item);
          }
        } else {
          this.process_message_from_server(data);
        }
      } catch (error) {
        console.error("Wrong response", event.data, error);
      }
    };

    this.ws.onopen = () => {
      console.log("^_^ Connected to server");

      // Try to authenticate user
      const token = localStorage.getItem("token");
      if (token) {
        this.auth(token);
      } else {
        this.triggerEvent("auth", false);
      }

      // Send queued requests
      this.queue.forEach((req) => {
        this.request({
          id: req.id,
          method: req.method,
          params: req.params,
        }).then(req.resolve, req.reject);
      });
    };
  }

  private check_connection() {
    if (this.lastState != this.ws.readyState) {
      this.lastState = this.ws.readyState;
      console.log(`••• Server state ${wsStateToString(this.ws.readyState)}`);
    }
    if (this._stop) return;
    if (this.ws.readyState === WebSocket.CLOSED) {
      // retry to reconnect in 5 seconds
      setTimeout(() => {
        this.ws = new WebSocket(`${import.meta.env.VITE_WS_SERVER}/ws`);
        this.init();
        if (this._stop) return;
        setTimeout(() => this.check_connection(), 5000);
      }, 5000);
    } else {
      setTimeout(() => this.check_connection(), 5000);
    }
  }

  private registerHandler(
    msg: string,
    handler: (data: any) => void,
    errorHandler: (data: any) => void,
  ) {
    // console.log(`Registering handler for ${msg}`);
    this.handlers.set(msg, handler);
    this.errorHandlers.set(msg, errorHandler);
  }

  private unregisterHandler(msg: string) {
    // console.log(`Unregistering handler for ${msg}`);
    this.handlers.delete(msg);
    this.errorHandlers.delete(msg);
  }

  // ---------------------------------------------------------------------------
  // Public Methods
  //

  stop() {
    console.log("Stopping server connection");
    this._stop = true;
  }

  async auth(token: string): Promise<boolean> {
    localStorage.setItem("token", token);
    return this.request<AuthenticateAPI>({
      id: "fake",
      method: "Authenticate",
      params: { token },
    }).then(
      (res) => {
        console.log("^_^ Authenticated", res);
        this.is_authenticated = true;
        this.triggerEvent("auth", true);
        return true;
      },
      (e) => {
        console.error(e);
        this.logout();
        return false;
      },
    );
  }

  logout() {
    this.is_authenticated = false;
    this.triggerEvent("auth", false);
    localStorage.removeItem("token");
  }

  // ---------------------------------------------------------------------------
  // -- Event Handlers
  addEventHandler(event: string, handler: EventHandler) {
    if (!this.eventHandlers.has(event)) {
      this.eventHandlers.set(event, []);
    }
    const handlers = this.eventHandlers.get(event);
    if (handlers) {
      handlers.push(handler);
    }
  }

  removeEventHandler(event: string, handler: EventHandler) {
    if (this.eventHandlers.has(event)) {
      const handlers = this.eventHandlers.get(event);
      if (handlers) {
        const index = handlers.indexOf(handler);
        if (index > -1) {
          handlers.splice(index, 1);
        }
      }
    }
  }

  triggerEvent(event: string, data: any) {
    if (this.eventHandlers.has(event)) {
      console.log("@@@", event, data);
      const handlers = this.eventHandlers.get(event);
      if (handlers) {
        handlers.forEach((handler) => handler(data));
      }
    }
  }

  // ---------------------------------------------------------------------------
  // -- Server Message Handlers
  addServerMessageHandler(key: string, handler: ServerMessageHandler) {
    if (!this.serverMessageHandlers.has(key)) {
      this.serverMessageHandlers.set(key, []);
    }
    const handlers = this.serverMessageHandlers.get(key);
    if (handlers) {
      handlers.push(handler);
    }
  }

  removeServerMessageHandler(key: string, handler: ServerMessageHandler) {
    if (this.serverMessageHandlers.has(key)) {
      const handlers = this.serverMessageHandlers.get(key);
      if (handlers) {
        const index = handlers.indexOf(handler);
        if (index > -1) {
          handlers.splice(index, 1);
        }
      }
    }
  }

  // ---------------------------------------------------------------------------
  // -- Request
  async request<T extends { i: JsonValue; o: JsonValue }>(
    payload: NitramRequest & { params: T["i"] },
  ) {
    let promise = new Promise<T["o"]>((resolve, reject) => {
      this.registerHandler(
        payload.method,
        (response: T["o"]) => {
          console.log("===", payload.method, response);
          resolve(response);
        },
        (error: string) => {
          console.error("===", payload.method, error);
          if (error === "(~ not authenticated ~)") {
            this.triggerEvent("(~ not authenticated ~)", null);
          }
          reject(error);
        },
      );
    });

    if (this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(payload));
      try {
        let res = await promise;
        return res;
      } catch (error) {
        throw error;
      } finally {
        this.unregisterHandler(payload.method);
      }
    } else {
      console.log("Queueing request", payload);
      let item: QueueItem = {
        id: payload.id,
        method: payload.method,
        params: payload.params,
        resolve: () => {},
        reject: () => {},
      };
      const promise = new Promise<T["o"]>((res, rej) => {
        item.resolve = res;
        item.reject = rej;
      });
      this.queue.push(item);
      return promise;
    }
  }
}
