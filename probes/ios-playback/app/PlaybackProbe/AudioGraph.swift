import AVFAudio

/// Replace the entire graph after a media-services reset. Creating a graph
/// attaches its nodes but never starts audio or activates the session.
@MainActor
struct ProbeAudioGraph {
    let engine: AVAudioEngine
    let source: AVAudioSourceNode

    init(render: @escaping AVAudioSourceNodeRenderBlock) {
        let engine = AVAudioEngine()
        let format = AVAudioFormat(standardFormatWithSampleRate: 44_100, channels: 2)!
        let source = AVAudioSourceNode(format: format, renderBlock: render)
        engine.attach(source)
        engine.connect(source, to: engine.mainMixerNode, format: format)
        self.engine = engine
        self.source = source
    }
}
