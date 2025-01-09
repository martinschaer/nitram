import { createStore } from "solid-js/store";

export const [messages, setMessages] = createStore<string[]>([]);
