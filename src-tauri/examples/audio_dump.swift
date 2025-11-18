@preconcurrency import AVFoundation
import CoreMedia
import Dispatch
import Foundation
@preconcurrency import ScreenCaptureKit

@main
struct SystemAudioDump {
  static func main() async {
    do {
      print("Starting SystemAudioDump...")

      // Parse CLI arguments
      let arguments = Array(CommandLine.arguments.dropFirst())
      var wavOutputURL: URL?
      var index = 0
      while index < arguments.count {
        let arg = arguments[index]
        if arg == "--wav" {
          let pathIndex = index + 1
          guard pathIndex < arguments.count else {
            fputs("Error: --wav requires a path argument\n", Darwin.stderr)
            exit(1)
          }
          wavOutputURL = URL(
            fileURLWithPath: (arguments[pathIndex] as NSString).expandingTildeInPath)
          index += 1
        } else if arg.hasPrefix("--wav=") {
          let path = String(arg.dropFirst("--wav=".count))
          if path.isEmpty {
            fputs("Error: --wav requires a path argument\n", Darwin.stderr)
            exit(1)
          }
          wavOutputURL = URL(fileURLWithPath: (path as NSString).expandingTildeInPath)
        }
        index += 1
      }
      if let wavURL = wavOutputURL {
        print("WAV output enabled: \(wavURL.path)")
      }

      // Check if we have screen recording permission
      print("Checking permissions...")
      let canRecord = CGPreflightScreenCaptureAccess()
      if !canRecord {
        print("❌ Screen recording permission required!")
        print("Please go to System Preferences > Security & Privacy > Privacy > Screen Recording")
        print("and enable access for this application.")

        // Request permission
        let granted = CGRequestScreenCaptureAccess()
        if !granted {
          print("Permission denied. Exiting.")
          exit(1)
        }
      }
      print("✅ Permissions OK")

      print("Getting shareable content...")
      let content = try await SCShareableContent.excludingDesktopWindows(
        false,
        onScreenWindowsOnly: true)
      guard let display = content.displays.first else {
        fatalError("No display found")
      }
      print("Found display: \(display)")

      // 2) Build a filter for that display (video is ignored below)
      let filter = SCContentFilter(
        display: display,
        excludingApplications: [],  // don't exclude any
        exceptingWindows: [])
      print("Created filter")

      // 3) Build a stream config that only captures audio
      let cfg = SCStreamConfiguration()
      cfg.capturesAudio = true
      cfg.captureMicrophone = false
      cfg.excludesCurrentProcessAudio = true  // don't capture our own output
      print("Created configuration")

      // 4) Create and start the stream
      let dumper = try AudioDumper(wavOutputURL: wavOutputURL)
      let stream = SCStream(
        filter: filter,
        configuration: cfg,
        delegate: dumper)
      print("Created stream")

      // only install audio output
      try stream.addStreamOutput(
        dumper,
        type: .audio,
        sampleHandlerQueue: DispatchQueue(label: "audio"))
      print("Added stream output")

      try await stream.startCapture()
      print("Started capture")

      await MainActor.run {
        print("✅ Capturing system audio. Press ⌃C to stop.", to: &standardError)
      }

      // keep the process alive with a safer approach
      print("Entering main loop...")

      // Set up signal handling for graceful shutdown
      signal(SIGINT, SIG_IGN)
      let sigintSource = DispatchSource.makeSignalSource(signal: SIGINT, queue: .main)
      sigintSource.setEventHandler {
        print("Received SIGINT, shutting down...")
        dumper.stop()
        exit(0)
      }
      sigintSource.resume()

      // Keep alive with a simple loop instead of dispatchMain
      while true {
        try await Task.sleep(nanoseconds: 1_000_000_000)  // 1 second
      }

    } catch {
      fputs("Error: \(error)\n", Darwin.stderr)
      exit(1)
    }
  }
}

/// A simple SCStreamOutput + SCStreamDelegate that converts to 16 kHz Int16 PCM and writes to stdout (and optional WAV file)
final class AudioDumper: NSObject, SCStreamDelegate, SCStreamOutput {
  // We'll hold a converter from native rate to 16 kHz, 16-bit, interleaved mono.
  private var converter: AVAudioConverter?
  private var outputFormat: AVAudioFormat?
  private let wavWriter: WAVFileWriter?
  private let targetSampleRate: Double = 16_000
  private let targetChannels: AVAudioChannelCount = 1

  init(wavOutputURL: URL?) throws {
    if let url = wavOutputURL {
      self.wavWriter = try WAVFileWriter(
        url: url,
        sampleRate: Int(targetSampleRate),
        channels: Int(targetChannels),
        bitsPerSample: 16)
    } else {
      self.wavWriter = nil
    }
    super.init()
  }

  func stop() {
    wavWriter?.finalize()
  }

  func stream(
    _ stream: SCStream,
    didOutputSampleBuffer sampleBuffer: CMSampleBuffer,
    of outputType: SCStreamOutputType
  ) {
    guard outputType == .audio else { return }

    // Wrap the CMSampleBuffer in an AudioBufferList
    do {
      try sampleBuffer.withAudioBufferList { abl, _ in
        guard let desc = sampleBuffer.formatDescription?.audioStreamBasicDescription else {
          return
        }

        // Initialize converter on first buffer
        if converter == nil {
          // source format
          guard
            let srcFormat = AVAudioFormat(
              commonFormat: .pcmFormatFloat32,
              sampleRate: desc.mSampleRate,
              channels: desc.mChannelsPerFrame,
              interleaved: false)
          else {
            fputs("Failed to create source format\n", Darwin.stderr)
            return
          }

          guard
            let targetFormat = AVAudioFormat(
              commonFormat: .pcmFormatInt16,
              sampleRate: targetSampleRate,
              channels: targetChannels,
              interleaved: true)
          else {
            fputs("Failed to create target format\n", Darwin.stderr)
            return
          }
          outputFormat = targetFormat
          converter = AVAudioConverter(from: srcFormat, to: targetFormat)
          print(
            """
            🔊 Audio Capture Format:
              Source: \(srcFormat.sampleRate) Hz, \(srcFormat.channelCount) channels, \(srcFormat.commonFormat == .pcmFormatFloat32 ? "Float32" : "Other")
              Target: \(targetFormat.sampleRate) Hz, \(targetFormat.channelCount) channels, \(targetFormat.commonFormat == .pcmFormatInt16 ? "Int16" : "Other")
            """)

        }

        guard let converter = converter,
          let outFmt = outputFormat
        else { return }

        // Create source AVAudioPCMBuffer
        let srcFmt = converter.inputFormat
        guard
          let srcBuffer = AVAudioPCMBuffer(
            pcmFormat: srcFmt,
            frameCapacity: AVAudioFrameCount(sampleBuffer.numSamples))
        else {
          return
        }
        srcBuffer.frameLength = srcBuffer.frameCapacity

        // Safely copy from AudioBufferList
        guard srcBuffer.floatChannelData != nil else { return }

        let channelCount = min(Int(srcFmt.channelCount), abl.count)
        for i in 0..<channelCount {
          guard i < abl.count,
            let channelData = srcBuffer.floatChannelData?[i],
            let bufferData = abl[i].mData
          else { continue }

          let bytesToCopy = min(
            Int(abl[i].mDataByteSize),
            Int(srcBuffer.frameCapacity) * MemoryLayout<Float>.size)
          memcpy(channelData, bufferData, bytesToCopy)
        }

        // Create output buffer with proper capacity calculation
        let outputFrameCapacity = AVAudioFrameCount(
          ceil(Double(srcBuffer.frameLength) * outFmt.sampleRate / srcFmt.sampleRate))
        guard
          let outBuffer = AVAudioPCMBuffer(
            pcmFormat: outFmt,
            frameCapacity: outputFrameCapacity)
        else {
          return
        }

        // Perform conversion
        var error: NSError?
        let status = converter.convert(
          to: outBuffer,
          error: &error
        ) { _, outStatus in
          outStatus.pointee = .haveData
          return srcBuffer
        }

        guard status != .error,
          outBuffer.frameLength > 0,
          let int16Data = outBuffer.int16ChannelData?[0]
        else {
          if let error = error {
            fputs("Conversion error: \(error)\n", Darwin.stderr)
          }
          return
        }

        // Write raw bytes to stdout
        let byteCount =
          Int(outBuffer.frameLength) * Int(outFmt.streamDescription.pointee.mBytesPerFrame)
        let data = Data(bytes: int16Data, count: byteCount)
        if let wavWriter {
          // 如果指定了 --wav，则只写入文件
          do {
            try wavWriter.append(data)
          } catch {
            fputs("Failed to write WAV data: \(error)\n", Darwin.stderr)
          }
        } else {
          // 否则写到 stdout
          FileHandle.standardOutput.write(data)
        }

      }
    } catch {
      fputs("Audio processing error: \(error)\n", Darwin.stderr)
    }
  }

  func stream(_ stream: SCStream, didStopWithError error: Error) {
    fputs("Stream stopped with error: \(error)\n", Darwin.stderr)
  }
}

final class WAVFileWriter {
  private let handle: FileHandle
  private let sampleRate: UInt32
  private let channels: UInt16
  private let bitsPerSample: UInt16
  private var dataBytesWritten: UInt64 = 0
  private var isClosed = false

  init(url: URL, sampleRate: Int, channels: Int, bitsPerSample: Int) throws {
    let fm = FileManager.default
    fm.createFile(atPath: url.path, contents: nil, attributes: nil)
    self.handle = try FileHandle(forWritingTo: url)
    self.sampleRate = UInt32(sampleRate)
    self.channels = UInt16(clamping: channels)
    self.bitsPerSample = UInt16(clamping: bitsPerSample)
    try writeHeader(dataSize: 0)
  }

  func append(_ data: Data) throws {
    guard !isClosed else { return }
    try handle.seekToEnd()
    try handle.write(contentsOf: data)
    dataBytesWritten &+= UInt64(data.count)
    try refreshHeader()
  }

  func finalize() {
    guard !isClosed else { return }
    do {
      try refreshHeader()
      try handle.close()
      isClosed = true
    } catch {
      fputs("Failed to finalize WAV file: \(error)\n", Darwin.stderr)
    }
  }

  private func refreshHeader() throws {
    try handle.seek(toOffset: 0)
    let header = Self.buildHeader(
      sampleRate: sampleRate,
      channels: channels,
      bitsPerSample: bitsPerSample,
      dataSize: UInt32(truncatingIfNeeded: dataBytesWritten))
    try handle.write(contentsOf: header)
    _ = try handle.seekToEnd()
  }

  private func writeHeader(dataSize: UInt32) throws {
    try handle.seek(toOffset: 0)
    let header = Self.buildHeader(
      sampleRate: sampleRate,
      channels: channels,
      bitsPerSample: bitsPerSample,
      dataSize: dataSize)
    try handle.write(contentsOf: header)
    _ = try handle.seekToEnd()
  }

  private static func buildHeader(
    sampleRate: UInt32,
    channels: UInt16,
    bitsPerSample: UInt16,
    dataSize: UInt32
  ) -> Data {
    var data = Data()
    data.append(contentsOf: "RIFF".utf8)
    var chunkSize = UInt32(36) &+ dataSize
    data.append(Data(bytes: &chunkSize, count: 4))
    data.append(contentsOf: "WAVE".utf8)
    data.append(contentsOf: "fmt ".utf8)
    var subchunk1Size: UInt32 = 16
    data.append(Data(bytes: &subchunk1Size, count: 4))
    var audioFormat: UInt16 = 1
    data.append(Data(bytes: &audioFormat, count: 2))
    var channelCount = channels
    data.append(Data(bytes: &channelCount, count: 2))
    var rate = sampleRate
    data.append(Data(bytes: &rate, count: 4))
    let byteRate = sampleRate * UInt32(channels) * UInt32(bitsPerSample) / 8
    var byteRateLE = byteRate
    data.append(Data(bytes: &byteRateLE, count: 4))
    let blockAlign = UInt16(channels) * bitsPerSample / 8
    var blockAlignLE = blockAlign
    data.append(Data(bytes: &blockAlignLE, count: 2))
    var bits = bitsPerSample
    data.append(Data(bytes: &bits, count: 2))
    data.append(contentsOf: "data".utf8)
    var subchunk2Size = dataSize
    data.append(Data(bytes: &subchunk2Size, count: 4))
    return data
  }
}

// Helper to print to stderr
@MainActor var standardError = FileHandle.standardError
extension FileHandle: @retroactive TextOutputStream {
  public func write(_ string: String) {
    if let data = string.data(using: .utf8) {
      self.write(data)
    }
  }
}
