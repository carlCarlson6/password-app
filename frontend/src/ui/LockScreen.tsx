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
    <main className="term-screen">
      <div className="term-window">
        <header className="term-titlebar">
          <h1 className="term-cursor">Vault locked</h1>
          <span aria-hidden="true">▓▒░</span>
        </header>
        <div className="term-body">
          <p className="text-sm text-crt-dim">Enter your master password to unlock.</p>
          <form
            onSubmit={(event) => void handleSubmit(event)}
            className="flex flex-col gap-4"
          >
            <label className="term-label">
              Master password
              <input
                className="term-input"
                type="password"
                autoComplete="current-password"
                autoFocus
                value={password}
                disabled={pending}
                onChange={(event) => setPassword(event.target.value)}
              />
            </label>
            <button type="submit" className="term-btn" disabled={pending || password === ""}>
              {pending ? "Unlocking…" : "Unlock"}
            </button>
            {failed && (
              <p role="alert" className="term-error">
                Wrong master password — try again.
              </p>
            )}
          </form>
        </div>
      </div>
    </main>
  );
}
