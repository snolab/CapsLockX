// clx-ocr — capture the frontmost normal-layer window and print OCR text.
//
// Usage: clx-ocr
// Prints Vision .accurate results (en+ja+zh) to stdout.
//
// Uses the `screencapture` CLI to grab the frontmost window by CGWindowID —
// `CGWindowListCreateImage` was removed in macOS 15 in favor of the async
// ScreenCaptureKit API. Shelling out keeps this a simple synchronous binary.
//
// Requires Screen Recording permission (already required by CLX's agent).
import AppKit
import CoreGraphics
import Foundation
import Vision

// ── 1. Find the active window ─────────────────────────────────────────
// Strategy: z-order enumerate on-screen layer-0 windows, but only accept
// the first one owned by NSWorkspace's frontmost application. This lines
// up the OCR target with the user's focused app even when a helper
// palette from another process sits in front.
let frontPid: pid_t = NSWorkspace.shared.frontmostApplication?.processIdentifier ?? 0
guard let arr = CGWindowListCopyWindowInfo(
    [.optionOnScreenOnly, .excludeDesktopElements],
    kCGNullWindowID
) as? [[String: Any]] else {
    FileHandle.standardError.write("clx-ocr: CGWindowListCopyWindowInfo nil\n".data(using: .utf8)!)
    exit(1)
}
var frontWid: CGWindowID = 0
var fallbackWid: CGWindowID = 0  // first size-ok layer-0 window, any owner
for w in arr {
    guard let layer = w[kCGWindowLayer as String] as? Int, layer == 0 else { continue }
    let b = w[kCGWindowBounds as String] as? [String: CGFloat] ?? [:]
    let wi = Int(b["Width"] ?? 0), hi = Int(b["Height"] ?? 0)
    if wi < 200 || hi < 150 { continue }
    guard let id = w[kCGWindowNumber as String] as? CGWindowID else { continue }
    if fallbackWid == 0 { fallbackWid = id }
    let ownerPid = (w[kCGWindowOwnerPID as String] as? pid_t) ?? 0
    if frontPid != 0 && ownerPid == frontPid {
        frontWid = id
        break
    }
}
// If frontmostApplication gave us nothing usable (e.g. Finder desktop),
// fall back to the plain z-order heuristic so we still OCR *something*.
if frontWid == 0 { frontWid = fallbackWid }
if frontWid == 0 {
    FileHandle.standardError.write("clx-ocr: no frontmost window found\n".data(using: .utf8)!)
    exit(2)
}

// ── 2. Capture window pixels via `screencapture -l <wid>` ──────────────
let tmpPath = NSTemporaryDirectory() + "clx-ocr-\(getpid()).png"
defer { try? FileManager.default.removeItem(atPath: tmpPath) }
let proc = Process()
proc.launchPath = "/usr/sbin/screencapture"
proc.arguments = ["-x", "-l", String(frontWid), tmpPath]
do {
    try proc.run()
    proc.waitUntilExit()
} catch {
    FileHandle.standardError.write("clx-ocr: screencapture failed: \(error)\n".data(using: .utf8)!)
    exit(3)
}
guard proc.terminationStatus == 0,
      let img = NSImage(contentsOfFile: tmpPath),
      let cg = img.cgImage(forProposedRect: nil, context: nil, hints: nil) else {
    FileHandle.standardError.write("clx-ocr: could not load captured image\n".data(using: .utf8)!)
    exit(4)
}

// ── 3. Vision OCR .accurate (CJK-capable) ─────────────────────────────
let req = VNRecognizeTextRequest()
req.recognitionLevel = .accurate
let supported = (try? req.supportedRecognitionLanguages()) ?? []
let wanted = ["en-US", "ja-JP", "zh-Hans"]
req.recognitionLanguages = wanted.filter { supported.contains($0) }
req.usesLanguageCorrection = true

let handler = VNImageRequestHandler(cgImage: cg, options: [:])
do {
    try handler.perform([req])
} catch {
    FileHandle.standardError.write("clx-ocr: vision error: \(error)\n".data(using: .utf8)!)
    exit(5)
}
for obs in (req.results ?? []) {
    if let top = obs.topCandidates(1).first {
        print(top.string)
    }
}
