import Foundation

/// C callbacks copy data on runtime threads, then deliver it on the main actor.
/// A ticket binds that delivery to one connection, including across Forget
/// followed immediately by a new sign-in. All shared state is under this lock.
public final class CredentialCallbackGate: @unchecked Sendable {
    private let lock = NSLock()
    private var generation: UInt64 = 0
    private var enabled = false

    public init() {}

    public func begin() {
        lock.lock(); defer { lock.unlock() }
        generation &+= 1
        enabled = true
    }

    public func revoke() {
        lock.lock(); defer { lock.unlock() }
        generation &+= 1
        enabled = false
    }

    public func ticket() -> UInt64? {
        lock.lock(); defer { lock.unlock() }
        return enabled ? generation : nil
    }

    public func accepts(_ ticket: UInt64) -> Bool {
        lock.lock(); defer { lock.unlock() }
        return enabled && generation == ticket
    }

    /// A synchronous C ABI rejection starts no task and emits no later event.
    public func connectionStarted(_ result: Int32) -> Bool {
        guard result == 0 else { revoke(); return false }
        return true
    }

}
