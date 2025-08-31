/* @refresh reload */
import type { MessagesAPI } from "bindings/API";
import { type EventHandler, Server } from "nitram";
import {
  type Accessor,
  createContext,
  createMemo,
  createSignal,
  type JSX,
  Match,
  onCleanup,
  onMount,
  type ParentComponent,
  Switch,
  useContext,
} from "solid-js";

// Local imports
import { messages, setMessages } from "./store";

// -----------------------------------------------------------------------------
// Handlers
//
export const messagesHandler = (channel: string, payload: MessagesAPI["o"]) => {
  if (Array.isArray(payload.messages)) {
    setMessages({ ...messages, [channel]: payload.messages });
  } else {
    console.error("Payload type is different than expected", payload);
  }
};

// -----------------------------------------------------------------------------
// Context Type
//
type BackendContextType = {
  server: Accessor<Server>;
};
export const BackendContext = createContext<BackendContextType>();

// =============================================================================
// Component
// =============================================================================
export const BackendProvider: ParentComponent<{
  publicChildren: JSX.Element;
}> = (props) => {
  // -- State
  const server = createMemo(() => new Server());
  const [isAuthenticated, isAuthenticatedSet] = createSignal<boolean | null>(
    null,
  );

  // -- Callbacks
  const pleaseLogInHandler: EventHandler = (_event: unknown) => {
    isAuthenticatedSet(false);
  };

  // -- Lifecycle
  onMount(() => {
    const _server = server();
    // We could register for Messages here, but we can do that in the Chat
    // component so we don't make unnecessary requests
    // _server.addServerMessageHandler("Messages", messagesHandler);
    _server.addEventHandler("(~ not authenticated ~)", pleaseLogInHandler);
    _server.addEventHandler("auth", isAuthenticatedSet);
  });

  onCleanup(() => {
    const _server = server();
    // _server.removeServerMessageHandler("Messages", messagesHandler);
    _server.removeEventHandler("(~ not authenticated ~)", pleaseLogInHandler);
    _server.removeEventHandler("auth", isAuthenticatedSet);
    _server.stop();
  });

  // -- Render
  return (
    <BackendContext.Provider value={{ server }}>
      <Switch>
        <Match when={isAuthenticated() === null}>Connecting...</Match>
        <Match when={isAuthenticated() === true}>{props.children}</Match>
        <Match when={isAuthenticated() === false}>{props.publicChildren}</Match>
      </Switch>
    </BackendContext.Provider>
  );
};

export function useBackend() {
  const context = useContext(BackendContext);
  if (!context) {
    throw new Error("useBackend must be used within a BackendProvider");
  }
  return context;
}
