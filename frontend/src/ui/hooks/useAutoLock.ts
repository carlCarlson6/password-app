import { useEffect } from "react";

import { DEFAULT_IDLE_LOCK_MS, makeIdleAutoLock } from "../../application/autoLock";

/** DOM events that count as "the user is here" for the idle countdown. */
const ACTIVITY_EVENTS = [
  "pointerdown",
  "pointermove",
  "keydown",
  "wheel",
  "touchstart",
  "scroll",
] as const;

interface UseAutoLockOptions {
  /** Arm only while unlocked — an armed timer on the lock screen is noise. */
  enabled: boolean;
  /** The lock use case (or anything that clears the key store). */
  onLock: () => void;
  idleMs?: number;
}

/**
 * Thin DOM adapter around the pure `makeIdleAutoLock` service: wires user
 * activity to the countdown reset, and locks immediately when the tab is
 * closed or navigated away (`pagehide`/`beforeunload`). In-memory keys vanish
 * on close anyway; clearing explicitly is defense in depth.
 */
export function useAutoLock({
  enabled,
  onLock,
  idleMs = DEFAULT_IDLE_LOCK_MS,
}: UseAutoLockOptions): void {
  useEffect(() => {
    if (!enabled) return;

    const autoLock = makeIdleAutoLock(onLock, idleMs);
    autoLock.start();

    const handleActivity = () => autoLock.recordActivity();
    for (const event of ACTIVITY_EVENTS) {
      window.addEventListener(event, handleActivity, { passive: true });
    }

    const handleLeave = () => onLock();
    window.addEventListener("pagehide", handleLeave);
    window.addEventListener("beforeunload", handleLeave);

    return () => {
      autoLock.stop();
      for (const event of ACTIVITY_EVENTS) {
        window.removeEventListener(event, handleActivity);
      }
      window.removeEventListener("pagehide", handleLeave);
      window.removeEventListener("beforeunload", handleLeave);
    };
  }, [enabled, onLock, idleMs]);
}
