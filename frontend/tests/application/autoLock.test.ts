// Tests for src/application/autoLock.ts — driven entirely with fake timers.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { DEFAULT_IDLE_LOCK_MS, makeIdleAutoLock } from "../../src/application/autoLock";

const IDLE_MS = 1000;

describe("makeIdleAutoLock", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("fires onIdle once the idle period elapses", () => {
    const onIdle = vi.fn();
    makeIdleAutoLock(onIdle, IDLE_MS).start();

    vi.advanceTimersByTime(IDLE_MS - 1);
    expect(onIdle).not.toHaveBeenCalled();

    vi.advanceTimersByTime(1);
    expect(onIdle).toHaveBeenCalledTimes(1);
  });

  it("does not fire before start() is called", () => {
    const onIdle = vi.fn();
    makeIdleAutoLock(onIdle, IDLE_MS);

    vi.advanceTimersByTime(IDLE_MS * 10);

    expect(onIdle).not.toHaveBeenCalled();
  });

  it("activity resets the countdown", () => {
    const onIdle = vi.fn();
    const autoLock = makeIdleAutoLock(onIdle, IDLE_MS);
    autoLock.start();

    // Keep poking just before the deadline: the lock must never fire.
    for (let i = 0; i < 5; i++) {
      vi.advanceTimersByTime(IDLE_MS - 1);
      autoLock.recordActivity();
    }
    expect(onIdle).not.toHaveBeenCalled();

    // Then go idle for the full period.
    vi.advanceTimersByTime(IDLE_MS);
    expect(onIdle).toHaveBeenCalledTimes(1);
  });

  it("stop() disarms the countdown", () => {
    const onIdle = vi.fn();
    const autoLock = makeIdleAutoLock(onIdle, IDLE_MS);
    autoLock.start();

    autoLock.stop();
    vi.advanceTimersByTime(IDLE_MS * 10);

    expect(onIdle).not.toHaveBeenCalled();
  });

  it("activity while stopped does not re-arm the countdown", () => {
    const onIdle = vi.fn();
    const autoLock = makeIdleAutoLock(onIdle, IDLE_MS);
    autoLock.start();
    autoLock.stop();

    autoLock.recordActivity();
    vi.advanceTimersByTime(IDLE_MS * 10);

    expect(onIdle).not.toHaveBeenCalled();
  });

  it("start() after firing arms a fresh countdown", () => {
    const onIdle = vi.fn();
    const autoLock = makeIdleAutoLock(onIdle, IDLE_MS);
    autoLock.start();
    vi.advanceTimersByTime(IDLE_MS);
    expect(onIdle).toHaveBeenCalledTimes(1);

    autoLock.start();
    vi.advanceTimersByTime(IDLE_MS);

    expect(onIdle).toHaveBeenCalledTimes(2);
  });

  it("defaults to a 5-minute idle period", () => {
    const onIdle = vi.fn();
    makeIdleAutoLock(onIdle).start();

    vi.advanceTimersByTime(DEFAULT_IDLE_LOCK_MS - 1);
    expect(onIdle).not.toHaveBeenCalled();

    vi.advanceTimersByTime(1);
    expect(onIdle).toHaveBeenCalledTimes(1);
    expect(DEFAULT_IDLE_LOCK_MS).toBe(5 * 60 * 1000);
  });
});
