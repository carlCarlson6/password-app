// @vitest-environment jsdom
// Tests for src/ui/SignupForm.tsx — above all: no submission without the
// explicit "no recovery in v1" acknowledgement.

import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { RegisterUser } from "../../src/application/registerUser";
import { SignupForm } from "../../src/ui/SignupForm";

afterEach(cleanup);

function fillCredentials() {
  fireEvent.change(screen.getByLabelText(/^email/i), {
    target: { value: "user@example.com" },
  });
  fireEvent.change(screen.getByLabelText(/^master password/i), {
    target: { value: "hunter2 but longer" },
  });
  fireEvent.change(screen.getByLabelText(/^confirm master password/i), {
    target: { value: "hunter2 but longer" },
  });
}

function renderForm(registerUser: RegisterUser = vi.fn(async () => {})) {
  const onRegistered = vi.fn();
  render(<SignupForm registerUser={registerUser} onRegistered={onRegistered} />);
  return { registerUser, onRegistered };
}

describe("SignupForm", () => {
  it("warns loudly that a forgotten master password is unrecoverable", () => {
    renderForm();

    const warnings = screen.getAllByRole("alert");
    expect(warnings[0].textContent).toMatch(/NO password recovery/i);
    expect(warnings[0].textContent).toMatch(/permanently unrecoverable/i);
  });

  it("blocks submission until the no-recovery acknowledgement is checked", async () => {
    const { registerUser, onRegistered } = renderForm();
    fillCredentials();

    // The submit button is disabled…
    expect(
      screen.getByRole<HTMLButtonElement>("button", { name: /create account/i })
        .disabled,
    ).toBe(true);

    // …and even a forced form submission is rejected by the handler.
    fireEvent.submit(screen.getByRole("form", { name: "sign up" }));

    await waitFor(() => {
      expect(screen.getByText(/must acknowledge/i)).toBeTruthy();
    });
    expect(registerUser).not.toHaveBeenCalled();
    expect(onRegistered).not.toHaveBeenCalled();
  });

  it("submits through the use case once acknowledged", async () => {
    const { registerUser, onRegistered } = renderForm();
    fillCredentials();
    fireEvent.click(screen.getByLabelText(/i understand/i));

    fireEvent.click(screen.getByRole("button", { name: /create account/i }));

    await waitFor(() => {
      expect(onRegistered).toHaveBeenCalledWith("user@example.com");
    });
    expect(registerUser).toHaveBeenCalledWith("user@example.com", "hunter2 but longer");
  });

  it("blocks mismatched passwords", async () => {
    const { registerUser } = renderForm();
    fillCredentials();
    fireEvent.change(screen.getByLabelText(/^confirm master password/i), {
      target: { value: "something else" },
    });
    fireEvent.click(screen.getByLabelText(/i understand/i));

    fireEvent.submit(screen.getByRole("form", { name: "sign up" }));

    await waitFor(() => {
      expect(screen.getByText(/do not match/i)).toBeTruthy();
    });
    expect(registerUser).not.toHaveBeenCalled();
  });

  it("surfaces a registration failure without clearing the form", async () => {
    const { registerUser } = renderForm(
      vi.fn(async () => {
        throw new Error("boom");
      }),
    );
    fillCredentials();
    fireEvent.click(screen.getByLabelText(/i understand/i));

    fireEvent.click(screen.getByRole("button", { name: /create account/i }));

    await waitFor(() => {
      expect(screen.getByText(/registration failed/i)).toBeTruthy();
    });
    expect(registerUser).toHaveBeenCalled();
    expect(screen.getByLabelText<HTMLInputElement>(/^email/i).value).toBe(
      "user@example.com",
    );
  });
});
