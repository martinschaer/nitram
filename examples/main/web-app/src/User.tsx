import type { GetUserAPI } from "bindings/API";
import type { User as UserModel } from "bindings/User";
import { createEffect, createSignal } from "solid-js";

// Local imports
import { useBackend } from "./BackendContext";

// =============================================================================
// Component
// =============================================================================
function User() {
  // -- State
  const [user, setUser] = createSignal<UserModel|null>(null);

  // -- Nitram context
  const { server } = useBackend();

  // -- Callbacks

  // -- Lifecycle
  createEffect(() => {
    const s = server();
    const user_id = s ? s.is_authenticated : null;
    if (user_id) {
      s.request<GetUserAPI>({
        method: "GetUser",
        params: {
          id: user_id
        }
      }).then((user) => {
        setUser(user);
      });
    }
  });

  // -- Render
  return (
    <span>
      {user()?.name}
    </span>
  );
}

export default User;
