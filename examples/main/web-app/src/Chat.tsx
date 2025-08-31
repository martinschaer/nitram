import type { MessagesAPI, SendMessageAPI } from "bindings/API";
import { NitramErrorCode, type ServerMessageHandler } from "nitram";
import { createMemo, createSignal, For, onMount } from "solid-js";

// Local imports
import { messagesHandler, useBackend } from "./BackendContext";
import { messages, setMessages } from "./store";
import User from "./User";

// =============================================================================
// Component
// =============================================================================
function Chat() {
  // -- HTML Elements
  let input!: HTMLInputElement;

  // -- State
  const [channel, setChannel] = createSignal("general");
  const channelMessages = createMemo(() => {
    return messages[channel()] ?? [];
  }, []);

  // -- Nitram context
  const { server } = useBackend();

  // -- Callbacks
  const handleLogout = () => {
    server().logout();
  };

  const handleMethod = () => {
    input.disabled = true;
    const _channel = channel();
    server()
      .request<SendMessageAPI>({
        method: "SendMessage",
        params: { channel: _channel, message: input.value },
      })
      .then((channel_messages) => {
        input.value = "";
        setMessages({ ...messages, [_channel]: channel_messages });
      })
      .catch((err) => {
        if (
          typeof err === "object" &&
          err.error === NitramErrorCode.DuplicateRequestQueued
        ) {
          input.value = "";
        } else {
          console.error("Error sending message:", err);
        }
      })
      .finally(() => {
        input.disabled = false;
      });
  };

  const channelHandler = (channel: string): ServerMessageHandler => {
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
      <button type="button" onClick={handleLogout}>
        Logout
      </button>
      <div>
        <h1>Chat</h1>
        <div>
          <User />:
          <input
            type="text"
            ref={(el) => {
              input = el;
            }}
            placeholder="Message"
          />
          <button type="button" onClick={handleMethod}>
            Send
          </button>
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
