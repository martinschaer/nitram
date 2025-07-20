/* @refresh reload */
import {
  Accessor,
  JSX,
  Match,
  ParentComponent,
  Switch,
  createContext,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";

// -----------------------------------------------------------------------------
// Nitram bindings
//
import { MessagesAPI } from "bindings/API";

// -----------------------------------------------------------------------------
// Local imports
//
import { Server } from "nitram";
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
  let server = createMemo(() => new Server());
  let [isAuthenticated, isAuthenticatedSet] = createSignal<boolean | null>(
    null,
  );

  // -- Callbacks
  const pleaseLogInHandler = (_event: string) => {
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
