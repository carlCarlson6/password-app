import { useState, type CSSProperties, type FormEvent } from "react";

import type { RegisterUser } from "../application/registerUser";

interface SignupFormProps {
  // Use case injected from the composition root — this component never
  // touches fetch or WebCrypto itself.
  registerUser: RegisterUser;
  onRegistered: (email: string) => void;
}

const warningStyle: CSSProperties = {
  border: "3px solid #c0392b",
  background: "#fdecea",
  color: "#7b241c",
  padding: "0.75rem 1rem",
  margin: "1rem 0",
  maxWidth: "34rem",
};

export function SignupForm({ registerUser, onRegistered }: SignupFormProps) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const [acknowledged, setAcknowledged] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const canSubmit =
    acknowledged &&
    !submitting &&
    email.length > 0 &&
    password.length > 0 &&
    confirmation.length > 0;

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    // The disabled button already blocks this; the guard keeps the invariant
    // even if the form is submitted some other way.
    if (!acknowledged) {
      setError("You must acknowledge that a lost master password cannot be recovered.");
      return;
    }
    if (password !== confirmation) {
      setError("The passwords do not match.");
      return;
    }
    setError(null);
    setSubmitting(true);
    try {
      await registerUser(email, password);
      onRegistered(email);
    } catch {
      setError("Registration failed. Please try again.");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <section>
      <h2>Create account</h2>

      <div role="alert" style={warningStyle}>
        <strong>⚠️ There is NO password recovery.</strong>
        <p>
          Your master password encrypts everything, and we never see it. If you
          forget it, your vault is <strong>permanently unrecoverable</strong> —
          no reset email, no support ticket, no exceptions. Write it down and
          store it somewhere safe before continuing.
        </p>
      </div>

      <form onSubmit={handleSubmit} aria-label="sign up">
        <div>
          <label>
            Email{" "}
            <input
              type="email"
              name="email"
              autoComplete="username"
              required
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
          </label>
        </div>
        <div>
          <label>
            Master password{" "}
            <input
              type="password"
              name="masterPassword"
              autoComplete="new-password"
              required
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
          </label>
        </div>
        <div>
          <label>
            Confirm master password{" "}
            <input
              type="password"
              name="masterPasswordConfirmation"
              autoComplete="new-password"
              required
              value={confirmation}
              onChange={(e) => setConfirmation(e.target.value)}
            />
          </label>
        </div>
        <div>
          <label>
            <input
              type="checkbox"
              name="acknowledgeNoRecovery"
              checked={acknowledged}
              onChange={(e) => setAcknowledged(e.target.checked)}
            />{" "}
            I understand that if I forget my master password, my vault is lost
            forever and cannot be recovered by anyone.
          </label>
        </div>
        <button type="submit" disabled={!canSubmit}>
          {submitting ? "Creating account…" : "Create account"}
        </button>
        {error && <p role="alert">{error}</p>}
      </form>
    </section>
  );
}
