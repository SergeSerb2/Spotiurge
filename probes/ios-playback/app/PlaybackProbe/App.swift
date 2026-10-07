import UIKit

/// Deliberately plain laboratory screen. It is not the Spotiurge iOS design.
@main
final class AppDelegate: UIResponder, UIApplicationDelegate {
    func application(_: UIApplication, didFinishLaunchingWithOptions _: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        Probe.shared.start()
        return true
    }
}

final class SceneDelegate: UIResponder, UIWindowSceneDelegate {
    var window: UIWindow?

    func scene(_ scene: UIScene, willConnectTo _: UISceneSession, options _: UIScene.ConnectionOptions) {
        guard let scene = scene as? UIWindowScene else { return }
        let window = UIWindow(windowScene: scene)
        window.rootViewController = ProbeViewController()
        window.makeKeyAndVisible()
        self.window = window
    }
}

final class ProbeViewController: UIViewController {
    private let status = UILabel()
    private let context = UITextField()
    private var signIn: SignIn?

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .systemBackground
        status.numberOfLines = 0
        status.font = .monospacedSystemFont(ofSize: 13, weight: .regular)
        context.borderStyle = .roundedRect
        context.autocapitalizationType = .none
        context.autocorrectionType = .no
        context.placeholder = "spotify:album:… or spotify:playlist:…"
        // Default: Daft Punk's Discovery, 14 tracks, checked through Spotify's public oEmbed.
        context.text = UserDefaults.standard.string(forKey: "context") ?? "spotify:album:2noRn2Aes5aoNVsU6iWThc"
        context.accessibilityLabel = "Context URI"

        let stack = UIStackView(arrangedSubviews: [
            status,
            button("Sign in to Spotify (streaming)") { [weak self] in self?.startSignIn() },
            context,
            button("Play context on this iPhone") { [weak self] in self?.play() },
            row(button("Play") { Probe.shared.command(0) }, button("Pause") { Probe.shared.command(1) }),
            row(button("Previous") { Probe.shared.command(3) }, button("Next") { Probe.shared.command(2) }),
            button("Take over from active Connect device") { Probe.shared.command(4) },
            button("Forget stored credential") { CredentialStore.delete() },
        ])
        stack.axis = .vertical
        stack.spacing = 12
        stack.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(stack)
        NSLayoutConstraint.activate([
            stack.leadingAnchor.constraint(equalTo: view.layoutMarginsGuide.leadingAnchor),
            stack.trailingAnchor.constraint(equalTo: view.layoutMarginsGuide.trailingAnchor),
            stack.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor, constant: 16),
            context.heightAnchor.constraint(greaterThanOrEqualToConstant: 44),
        ])
        Probe.shared.onChange = { [weak self] in self?.refresh() }
        refresh()
    }

    private func refresh() {
        status.text = Probe.shared.statusText()
    }

    private func play() {
        let uri = Self.contextURI(context.text ?? "")
        guard uri.hasPrefix("spotify:") else { return }
        UserDefaults.standard.set(uri, forKey: "context")
        let result = Probe.shared.load(uri)
        Probe.shared.record(["t": "load_requested", "uri": uri, "result": Int(result)])
        if result == -2 { Probe.shared.failed("play", "not connected to Spotify yet") }
    }

    /// Accepts a URI or an open.spotify.com share link.
    static func contextURI(_ text: String) -> String {
        let text = text.trimmingCharacters(in: .whitespacesAndNewlines)
        guard let url = URL(string: text), url.host == "open.spotify.com" else { return text }
        let parts = url.pathComponents.filter { $0 != "/" && !$0.hasPrefix("intl-") }
        return parts.count >= 2 ? "spotify:\(parts[0]):\(parts[1])" : text
    }

    private func startSignIn() {
        guard let window = view.window else { return }
        Probe.shared.record(["t": "sign_in_started"])
        let signIn = SignIn(anchor: window)
        self.signIn = signIn
        signIn.start { [weak self] result in
            switch result {
            case .success(let token):
                Probe.shared.connect(token: token)
            case .failure(let error):
                Probe.shared.failed("sign_in", error.localizedDescription)
            }
            self?.signIn = nil
            self?.refresh()
        }
    }

    private func button(_ title: String, action: @escaping () -> Void) -> UIButton {
        var configuration = UIButton.Configuration.filled()
        configuration.title = title
        let button = UIButton(configuration: configuration, primaryAction: UIAction { _ in
            action()
        })
        button.heightAnchor.constraint(greaterThanOrEqualToConstant: 44).isActive = true
        return button
    }

    private func row(_ views: UIView...) -> UIStackView {
        let row = UIStackView(arrangedSubviews: views)
        row.spacing = 12
        row.distribution = .fillEqually
        return row
    }
}
