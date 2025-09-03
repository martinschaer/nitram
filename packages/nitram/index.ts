import type { AuthenticateAPI } from "./bindings/API";
import type { NitramRequest } from "./bindings/NitramRequest";
import type { NitramResponse } from "./bindings/NitramResponse";
import type { NitramServerMessage } from "./bindings/NitramServerMessage";
import type { JsonValue } from "./bindings/serde_json/JsonValue";
import { NitramError, NitramErrorCode } from "./error";
import { objectHash } from "./hash";

export { NitramError, NitramErrorCode };

// biome-ignore lint/suspicious/noExplicitAny: see below what didn't work
type Handler = (x: any) => void;
// These didn't work:
// type Handler = <T extends JsonValue>(x: T) => void;
// type Handler = (x: JsonValue) => void;
// type Handler = (x: unknown) => void;

export type EventHandler = Handler;
export type ServerMessageHandler = Handler;
type QueueItem = NitramRequest & {
  hash: number;
  resolve: Handler;
  reject: (e: unknown) => void;
};
type HandlerByRequestId = Map<string, Handler>;

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

function randomId(length = 6) {
  return Math.random()
    .toString(36)
    .substring(2, length + 2);
}

// =============================================================================
// Server
// =============================================================================
export class Server {
  // -- Public
  is_authenticated: string | null = null;

  // -- Private
  private _stop = false;
  private lastState: number = WebSocket.CLOSED;
  private ws: WebSocket;
  private handlers: HandlerByRequestId = new Map();
  private errorHandlers: Map<string, (data: JsonValue) => void> = new Map();
  private eventHandlers: Map<string, EventHandler[]> = new Map();
  private serverMessageHandlers: Map<string, ServerMessageHandler[]> =
    new Map();
  private queue: QueueItem[] = [];

  // -- Constructor
  constructor() {
    this.ws = new WebSocket(`${import.meta.env.VITE_WS_SERVER}/ws`);
    this.init();
    this.check_connection();
  }

  // ---------------------------------------------------------------------------
  // Private Methods
  //

  private process_message_from_server(data: JsonValue) {
    if (data === null) {
      // - null
    } else if (typeof data === "object" && Object.hasOwn(data, "topic")) {
      // - server messages
      const serverMessageData = data as NitramServerMessage;

      // -- find registered server message handlers
      const handlers = this.serverMessageHandlers.get(serverMessageData.topic);
      if (handlers) {
        console.log(`<-- server msg: ${serverMessageData.topic}`);
        for (const handler of handlers) {
          handler(serverMessageData.payload);
        }
      } else {
        // -- unhandled server message
        console.log("<-- server msg unhandled: ", serverMessageData.topic);
      }
    } else {
      // - message responses
      if (
        typeof data === "object" &&
        Object.hasOwn(data, "method") &&
        Object.hasOwn(data, "ok") &&
        Object.hasOwn(data, "response")
      ) {
        const messageData = data as unknown as NitramResponse;
        if (messageData.ok) {
          const handler = this.handlers.get(messageData.id);
          if (handler) handler(messageData.response);
          else console.warn("!!! Unhandled message", messageData);
        } else {
          const handler = this.errorHandlers.get(messageData.id);
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
          method: req.method,
          params: req.params,
        }).then(req.resolve, req.reject);
      });
    };
  }

  private check_connection() {
    if (this.lastState !== this.ws.readyState) {
      this.lastState = this.ws.readyState;
      console.log(`••• Server state ${wsStateToString(this.ws.readyState)}`);
    }
    if (this._stop) return;
    if (this.ws.readyState === WebSocket.CLOSED) {
      // retry to reconnect in 5 seconds
      setTimeout(() => {
        this.ws = new WebSocket(`${import.meta.env.VITE_WS_SERVER}/ws`);
        // TODO: do we need to call init again?
        // this.init();
        if (this._stop) return;
        setTimeout(() => this.check_connection(), 5000);
      }, 5000);
    } else {
      setTimeout(() => this.check_connection(), 5000);
    }
  }

  private registerHandler(
    requestId: string,
    handler: Handler,
    errorHandler: Handler,
  ) {
    // console.log(`Registering handler for ${msg}`);
    this.handlers.set(requestId, handler);
    this.errorHandlers.set(requestId, errorHandler);
  }

  private unregisterHandler(requestId: string) {
    // console.log(`Unregistering handler for ${requestId}`);
    this.handlers.delete(requestId);
    this.errorHandlers.delete(requestId);
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
      method: "Authenticate",
      params: { token },
    }).then(
      (user_id) => {
        console.log("^_^ Authenticated", user_id);
        this.is_authenticated = user_id;
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
    this.is_authenticated = null;
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

  triggerEvent(event: string, data: JsonValue) {
    if (this.eventHandlers.has(event)) {
      console.log("@@@", event, data);
      const handlers = this.eventHandlers.get(event);
      if (handlers) {
        handlers.forEach((handler) => {
          handler(data);
        });
      }
    }
  }

  // ---------------------------------------------------------------------------
  // -- Server Message Handlers
  addServerMessageHandler(
    key: string,
    handler: ServerMessageHandler,
    params: { [key in string]?: JsonValue },
  ) {
    if (!this.serverMessageHandlers.has(key)) {
      this.serverMessageHandlers.set(key, []);
    }
    const handlers = this.serverMessageHandlers.get(key);
    if (handlers) {
      handlers.push(handler);
    }
    this.request({
      method: "nitram_topic_register",
      params: { topic: key, handler_params: params },
    });
  }

  removeServerMessageHandler(key: string) {
    if (this.serverMessageHandlers.has(key)) {
      this.serverMessageHandlers.delete(key);
    }
    this.request({
      method: "nitram_topic_deregister",
      params: { topic: key },
    });
  }

  // ---------------------------------------------------------------------------
  // -- Request
  async request<T extends { i: JsonValue; o: JsonValue }>(req: {
    method: string;
    params: T["i"];
  }) {
    const request_id = randomId();
    const payload: NitramRequest = {
      id: request_id,
      method: req.method,
      params: req.params,
    };

    if (this.ws.readyState === WebSocket.OPEN) {
      // Connection open -------------------------------------------------------
      const promise = new Promise<T["o"]>((resolve, reject) => {
        this.registerHandler(
          request_id,
          (response: T["o"]) => {
            console.log("===", req.method, response);
            resolve(response);
          },
          (error) => {
            console.error("===", req.method, error);
            if (error === "(~ not authenticated ~)") {
              this.triggerEvent("(~ not authenticated ~)", null);
            }
            reject(error);
          },
        );
      });
      this.ws.send(JSON.stringify(payload));
      try {
        const res = await promise;
        return res;
      } finally {
        this.unregisterHandler(request_id);
      }
    } else {
      // Connection closed -----------------------------------------------------
      const hash = objectHash({
        method: payload.method,
        params: payload.params,
      });
      // check if there is already a request with the same hash in the queue
      const existing = this.queue.find((item) => item.hash === hash);
      if (!existing) {
        console.log("Queueing request", payload);
        const item: QueueItem = {
          id: request_id,
          hash,
          method: payload.method,
          params: payload.params,
          resolve: (_) => {},
          reject: () => {},
        };
        const promise = new Promise<T["o"]>((res, rej) => {
          item.resolve = res;
          item.reject = rej;
        });
        this.queue.push(item);
        return promise;
      } else {
        // return error telling the caller that an identical request is already
        // queued
        return Promise.reject(
          new NitramError(NitramErrorCode.DuplicateRequestQueued, {
            id: existing.id,
          }),
        );
      }
    }
  }
}
