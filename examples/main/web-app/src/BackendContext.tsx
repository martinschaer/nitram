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
// Local imports
//
import { Server } from "./lib/nitram";
// import { signalsStore } from "./stores/signals";
import { AuthenticateAPI } from "nitram/API";

// -----------------------------------------------------------------------------
// Utils
//
const signalsHandler = (data: any) => {
  if (Array.isArray(data)) {
    for (let _d of data) {
      if (Object.hasOwn(_d, "progress") && Object.hasOwn(_d, "id")) {
        console.log(_d);
      } else {
        console.error("Invalid job data", _d);
      }
    }
  } else {
    console.error("Invalid job data", data);
  }
};

// -----------------------------------------------------------------------------
// Context Type
//
type BackendContextType = {
  server: Accessor<Server<AuthenticateAPI>>;
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
    _server.addSignalHandler("Signal", signalsHandler);
    _server.addEventHandler("(~ not authenticated ~)", pleaseLogInHandler);
    _server.addEventHandler("auth", isAuthenticatedSet);
  });

  onCleanup(() => {
    const _server = server();
    _server.removeSignalHandler("Signal", signalsHandler);
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
