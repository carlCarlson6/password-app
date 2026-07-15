import { useState, type FormEvent } from "react";

import type { Login } from "../application/login";

interface LoginFormProps {
  // Use case injected from the composition root — this component never
  // touches fetch or WebCrypto itself.
  login: Login;
  onLoggedIn: (email: string) => void;
}

export function LoginForm({ login, onLoggedIn }: LoginFormProps) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    setError(null);
    setSubmitting(true);
    try {
      await login(email, password);
      onLoggedIn(email);
    } catch {
      // Deliberately vague: no hint whether the email exists or which step failed.
      setError("Login failed. Check your email and master password.");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <section className="flex flex-col gap-4">
      <h2 className="term-heading">Log in</h2>
      <form onSubmit={handleSubmit} aria-label="log in" className="flex flex-col gap-4">
        <div className="term-field">
          <label className="term-label">
            Email
            <input
              className="term-input"
              type="email"
              name="email"
              autoComplete="username"
              required
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
          </label>
        </div>
        <div className="term-field">
          <label className="term-label">
            Master password
            <input
              className="term-input"
              type="password"
              name="masterPassword"
              autoComplete="current-password"
              required
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
          </label>
        </div>
        <button
          type="submit"
          className="term-btn"
          disabled={submitting || email.length === 0 || password.length === 0}
        >
          {submitting ? "Unlocking…" : "Log in"}
        </button>
        {error && (
          <p role="alert" className="term-error">
            {error}
          </p>
        )}
      </form>
    </section>
  );
}
