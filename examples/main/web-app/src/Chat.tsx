import { createMemo, createSignal, For, onMount, useContext } from "solid-js";

// -----------------------------------------------------------------------------
// Nitram bindings
//
import { MessagesAPI, SendMessageAPI } from "bindings/API";

// -----------------------------------------------------------------------------
// Local imports
//
import { BackendContext, messagesHandler } from "./BackendContext";
import { messages, setMessages } from "./store";
import User from "./User";

// =============================================================================
// Component
// =============================================================================
function Chat() {
  // -- HTML Elements
  let input!: HTMLInputElement;

  // -- State
  let [channel, setChannel] = createSignal("general");
  let channelMessages = createMemo(() => {
    return messages[channel()] ?? [];
  }, []);

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
    const _channel = channel();
    server()
      .request<SendMessageAPI>({
        id: "fake",
        method: "SendMessage",
        params: { channel: _channel, message: input.value },
      })
      .then((channel_messages) => {
        input.value = "";
        setMessages({ ...messages, [_channel]: channel_messages });
      })
      .finally(() => {
        input.disabled = false;
      });
  };

  const channelHandler = (channel: string) => {
    return (data: MessagesAPI["o"]) => messagesHandler(channel, data);
  };

  const changeChannel = (newChannel: string) => {
    server().removeServerMessageHandler("Messages");
    setChannel(newChannel);
    server().addServerMessageHandler("Messages", channelHandler(newChannel), {
      channel: newChannel,
    });
  };

  // -- Lifecycle
  onMount(() => changeChannel(channel()));

  // -- Render
  return (
    <>
      <button onClick={handleLogout}>Logout</button>
      <div>
        <h1>Chat</h1>
        <div>
          <User />:
          <input type="text" ref={(el) => (input = el)} placeholder="Message" />
          <button onClick={handleMethod}>Send</button>
        </div>
      </div>
      <div>
        <h2>Messages</h2>
        <select onChange={(e) => changeChannel(e.target.value)}>
          <option value="general">General</option>
          <option value="random">Random</option>
        </select>
        <ul>
          <For each={channelMessages()}>{(msg, _i) => <li>{msg}</li>}</For>
        </ul>
      </div>
    </>
  );
}

export default Chat;
