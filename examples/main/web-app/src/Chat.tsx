import { For, useContext } from "solid-js";

// -----------------------------------------------------------------------------
// Nitram bindings
//
import { SendMessageAPI } from "nitram/API";

// -----------------------------------------------------------------------------
// Local imports
//
import { BackendContext } from "./BackendContext";
import { messages } from "./store";

// =============================================================================
// Component
// =============================================================================
function Chat() {
  // -- HTML Elements
  let input!: HTMLInputElement;

  // -- Nitram context
  const { server } = useContext(BackendContext) ?? { server: null };
  if (!server) {
    throw new Error("BackendContext not found");
  }

  // -- Callbacks
  const handleLogout = () => {
    server().logout();
  };

  const handleMethod = () => {
    input.disabled = true;
    server()
      .request<SendMessageAPI>({
        id: "fake",
        method: "SendMessage",
        params: { message: input.value },
      })
      .then(() => {
        input.value = "";
      })
      .finally(() => {
        input.disabled = false;
      });
  };

  // -- Render
  return (
    <>
      <button onClick={handleLogout}>Logout</button>
      <div>
        <h1>Chat</h1>
        <input type="text" ref={(el) => (input = el)} placeholder="Message" />
        <button onClick={handleMethod}>Send</button>
      </div>
      <div>
        <h2>Messages</h2>
        <ul>
          <For each={messages}>{(msg, _i) => <li>{msg}</li>}</For>
        </ul>
      </div>
    </>
  );
}

export default Chat;
