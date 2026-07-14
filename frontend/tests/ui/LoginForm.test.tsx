// @vitest-environment jsdom
// Tests for src/ui/LoginForm.tsx.

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { Login } from "../../src/application/login";
import { LoginForm } from "../../src/ui/LoginForm";

afterEach(cleanup);

function fillAndSubmit() {
  fireEvent.change(screen.getByLabelText(/^email/i), {
    target: { value: "user@example.com" },
  });
  fireEvent.change(screen.getByLabelText(/^master password/i), {
    target: { value: "hunter2 but longer" },
  });
  fireEvent.click(screen.getByRole("button", { name: /log in/i }));
}

describe("LoginForm", () => {
  it("logs in through the use case and reports success", async () => {
    const login: Login = vi.fn(async () => {});
    const onLoggedIn = vi.fn();
    render(<LoginForm login={login} onLoggedIn={onLoggedIn} />);

    fillAndSubmit();

    await waitFor(() => {
      expect(onLoggedIn).toHaveBeenCalledWith("user@example.com");
    });
    expect(login).toHaveBeenCalledWith("user@example.com", "hunter2 but longer");
  });

  it("shows a deliberately vague error when login fails", async () => {
    const login: Login = vi.fn(async () => {
      throw new Error("invalid credentials");
    });
    const onLoggedIn = vi.fn();
    render(<LoginForm login={login} onLoggedIn={onLoggedIn} />);

    fillAndSubmit();

    await waitFor(() => {
      expect(screen.getByRole("alert").textContent).toMatch(/login failed/i);
    });
    // The exact failure ("invalid credentials") must not leak to the screen.
    expect(screen.queryByText(/invalid credentials/i)).toBeNull();
    expect(onLoggedIn).not.toHaveBeenCalled();
  });

  it("keeps the button disabled until both fields are filled", () => {
    render(<LoginForm login={vi.fn(async () => {})} onLoggedIn={vi.fn()} />);

    expect(
      screen.getByRole<HTMLButtonElement>("button", { name: /log in/i }).disabled,
    ).toBe(true);
  });
});
