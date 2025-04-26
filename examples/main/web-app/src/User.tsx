import { createSignal, useContext, createEffect } from "solid-js";

// -----------------------------------------------------------------------------
// Nitram bindings
//
import { GetUserAPI } from "bindings/API";
import { User as UserModel } from "bindings/User";

// -----------------------------------------------------------------------------
// Local imports
//
import { BackendContext } from "./BackendContext";

// =============================================================================
// Component
// =============================================================================
function User() {
  // -- State
  const [user, setUser] = createSignal<UserModel|null>(null);

  // -- Nitram context
  const { server } = useContext(BackendContext) ?? { server: null };
  if (!server) {
    throw new Error("BackendContext not found");
  }

  // -- Callbacks

  // -- Lifecycle
  createEffect(() => {
    const s = server();
    const user_id = s ? s.is_authenticated : null;
    if (user_id) {
      s.request<GetUserAPI>({
        id: "fake",
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
