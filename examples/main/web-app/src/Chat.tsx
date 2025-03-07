import { createSignal, For, useContext } from "solid-js";

// -----------------------------------------------------------------------------
// Nitram bindings
//
import { SendMessageAPI } from "nitram/API";

// -----------------------------------------------------------------------------
// Local imports
//
import { BackendContext, messagesHandler } from "./BackendContext";
import { messages, setMessages } from "./store";

// =============================================================================
// Component
// =============================================================================
function Chat() {
  // -- HTML Elements
  let input!: HTMLInputElement;

  // -- State
  let [listening, setListening] = createSignal(false);

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
      .then((messages) => {
        input.value = "";
        setMessages(messages);
      })
      .finally(() => {
        input.disabled = false;
      });
  };

  const handlePause = () => {
    const curr = listening();
    if (curr) {
      server().removeServerMessageHandler("Messages", messagesHandler);
    } else {
      server().addServerMessageHandler("Messages", messagesHandler);
    }
    setListening(!curr);
  };

  // -- Render
  return (
    <>
      <button onClick={handleLogout}>Logout</button>
      <div>
        <h1>Chat</h1>
        <input type="text" ref={(el) => (input = el)} placeholder="Message" />
        <button onClick={handleMethod}>Send</button>
        <button onClick={handlePause}>
          {listening() ? "Pause" : "Resume"}
        </button>
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
