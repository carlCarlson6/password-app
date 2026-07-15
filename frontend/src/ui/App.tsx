import { useEffect, useState } from "react";

import type { CheckServerHealth } from "../application/checkServerHealth";
import type { Login } from "../application/login";
import type { RegisterUser } from "../application/registerUser";
import { type ServerHealth, isHealthy } from "../domain/health";
import { LoginForm } from "./LoginForm";
import { SignupForm } from "./SignupForm";

interface AppProps {
  // Use cases arrive as props from the composition root (main.tsx):
  // the UI layer never builds gateways or touches fetch itself.
  checkServerHealth: CheckServerHealth;
  registerUser: RegisterUser;
  login: Login;
}

type View = "login" | "signup" | "unlocked";

export function App({ checkServerHealth, registerUser, login }: AppProps) {
  const [health, setHealth] = useState<ServerHealth | null>(null);
  const [view, setView] = useState<View>("login");
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void checkServerHealth().then((result) => {
      if (!cancelled) setHealth(result);
    });
    return () => {
      cancelled = true;
    };
  }, [checkServerHealth]);

  return (
    <main className="term-screen">
      <div className="term-window">
        <header className="term-titlebar">
          <h1 className="term-cursor">Password App</h1>
          <span aria-hidden="true">▓▒░</span>
        </header>

        <div className="term-body">
          {view !== "unlocked" && (
            <nav className="term-tabs" aria-label="authentication views">
              <button
                type="button"
                className="term-tab"
                onClick={() => {
                  setView("login");
                  setNotice(null);
                }}
                disabled={view === "login"}
              >
                Log in
              </button>
              <button
                type="button"
                className="term-tab"
                onClick={() => {
                  setView("signup");
                  setNotice(null);
                }}
                disabled={view === "signup"}
              >
                Sign up
              </button>
            </nav>
          )}

          {notice && (
            <p role="status" className="term-status">
              {notice}
            </p>
          )}

          {view === "login" && (
            <LoginForm
              login={login}
              onLoggedIn={() => {
                setNotice(null);
                setView("unlocked");
              }}
            />
          )}
          {view === "signup" && (
            <SignupForm
              registerUser={registerUser}
              onRegistered={() => {
                setNotice("Account created. Log in with your master password.");
                setView("login");
              }}
            />
          )}
          {view === "unlocked" && (
            <p role="status" className="term-status">
              Vault unlocked. Your keys are held in memory only.
            </p>
          )}
        </div>

        {health !== null && (
          <footer className="term-statusline">
            <small>
              Server is {isHealthy(health) ? "healthy" : "degraded"} — database{" "}
              {health.database}.
            </small>
          </footer>
        )}
      </div>
    </main>
  );
}
