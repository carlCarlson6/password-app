import { useState, type FormEvent } from "react";

interface LockScreenProps {
  /**
   * The unlock use case with its `UnlockContext` already applied by the
   * composition root — the UI only supplies the master password and never
   * touches WebCrypto itself. Resolves `false` on a wrong password.
   */
  unlock: (masterPassword: string) => Promise<boolean>;
}

export function LockScreen({ unlock }: LockScreenProps) {
  const [password, setPassword] = useState("");
  const [pending, setPending] = useState(false);
  const [failed, setFailed] = useState(false);

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (pending || password === "") return;

    setPending(true);
    setFailed(false);
    try {
      const unlocked = await unlock(password);
      setFailed(!unlocked);
    } catch {
      setFailed(true);
    } finally {
      // Drop the password from React state either way; on success the
      // component unmounts, on failure the user retypes from scratch.
      setPassword("");
      setPending(false);
    }
  };

  return (
    <main>
      <h1>Vault locked</h1>
      <p>Enter your master password to unlock.</p>
      <form onSubmit={(event) => void handleSubmit(event)}>
        <label>
          Master password
          <input
            type="password"
            autoComplete="current-password"
            autoFocus
            value={password}
            disabled={pending}
            onChange={(event) => setPassword(event.target.value)}
          />
        </label>
        <button type="submit" disabled={pending || password === ""}>
          {pending ? "Unlocking…" : "Unlock"}
        </button>
        {failed && <p role="alert">Wrong master password — try again.</p>}
      </form>
    </main>
  );
}
