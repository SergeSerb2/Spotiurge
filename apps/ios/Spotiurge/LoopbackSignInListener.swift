import Foundation
import Network

/// Owns one callback listener. Authentication may open only after its socket
/// is ready; callbacks queued by a canceled listener cannot affect its replacement.
@MainActor
final class LoopbackSignInListener {
    private var listener: NWListener?
    private var ready = false

    var port: UInt16? { listener?.port?.rawValue }

    func start(port: UInt16, onReady: @escaping @MainActor @Sendable () -> Void,
               onConnection: @escaping @MainActor @Sendable (NWConnection) -> Void,
               onFailure: @escaping @MainActor @Sendable () -> Void) {
        cancel()
        do {
            let parameters = NWParameters.tcp
            parameters.requiredLocalEndpoint = .hostPort(host: "127.0.0.1", port: NWEndpoint.Port(rawValue: port)!)
            parameters.allowLocalEndpointReuse = true
            let listener = try NWListener(using: parameters)
            self.listener = listener
            listener.stateUpdateHandler = { [weak self, weak listener] state in
                Task { @MainActor in
                    guard let self, let listener, self.listener === listener else { return }
                    switch state {
                    case .ready:
                        guard !self.ready else { return }
                        self.ready = true
                        onReady()
                    case .failed, .waiting:
                        // A loopback bind failure cannot be repaired by opening
                        // the browser. Close this attempt and allow a fresh retry.
                        self.cancel()
                        onFailure()
                    default: break
                    }
                }
            }
            listener.newConnectionHandler = { [weak self, weak listener] connection in
                Task { @MainActor in
                    guard let self, let listener, self.listener === listener else {
                        connection.cancel()
                        return
                    }
                    onConnection(connection)
                }
            }
            listener.start(queue: .main)
        } catch { onFailure() }
    }

    func cancel() {
        let old = listener
        listener = nil
        ready = false
        old?.cancel()
    }
}
