//! `clx ocr` — OCR a screenshot or image file using Apple Vision Framework.
//!
//! Usage:
//!   clx ocr                          # full screen, JSON output
//!   clx ocr --region x,y,w,h        # screen region
//!   clx ocr <image.png>              # OCR an existing image file
//!   clx ocr --text                   # plain text output (no JSON)
//!   clx ocr --min-confidence 0.5     # filter low-confidence results
//!
//! Output JSON fields (all coordinates in logical screen points, top-left origin,
//! ready for `m cx cy c`):
//!   text, x, y, w, h, cx, cy, confidence
//!
//! NOTE: This is a one-shot subprocess (~300ms cold). It belongs in the slow
//! layer (cheaper alternative to VLM, not a 20ms fast-layer call).
//! For fast-layer use, convert to a persistent helper process.

/// Python script embedded inline.
/// Coordinate conversion:
///   screencapture PNG has DPI metadata (144dpi on Retina, 72dpi on non-Retina).
///   NSImage.size() reads that metadata and returns logical points — no manual
///   scale calculation needed.
///   Vision bbox: normalized [0,1], bottom-left origin → flip y for top-left.
///   x_logical = region_x + x_norm  * img_logical_w
///   y_logical = region_y + (1 - y_norm - h_norm) * img_logical_h
const OCR_PYTHON: &str = r#"
import sys, os, json
import Vision
import AppKit
import Quartz

def main():
    img_path  = sys.argv[1] if len(sys.argv) > 1 else None
    region_x  = int(sys.argv[2]) if len(sys.argv) > 2 else 0
    region_y  = int(sys.argv[3]) if len(sys.argv) > 3 else 0

    if not img_path or not os.path.exists(img_path):
        print(json.dumps([]))
        return

    # NSImage.size() returns logical points (DPI-aware from PNG metadata).
    # screencapture embeds 144dpi on Retina, so size() == logical screen coords.
    ns_img = AppKit.NSImage.alloc().initWithContentsOfFile_(img_path)
    if ns_img is None:
        print(json.dumps({"error": "NSImage load failed"}))
        return
    logical_w = ns_img.size().width
    logical_h = ns_img.size().height

    # Vision needs a CFURL
    encoded = img_path.encode("utf-8")
    url = Quartz.CFURLCreateFromFileSystemRepresentation(None, encoded, len(encoded), False)

    # Vision OCR request
    handler = Vision.VNImageRequestHandler.alloc().initWithURL_options_(url, {})
    request = Vision.VNRecognizeTextRequest.alloc().init()
    request.setRecognitionLevel_(Vision.VNRequestTextRecognitionLevelAccurate)
    request.setUsesLanguageCorrection_(True)
    handler.performRequests_error_([request], None)

    results = []
    for obs in (request.results() or []):
        candidates = obs.topCandidates_(1)
        if not candidates:
            continue
        cand = candidates[0]
        text = cand.string()
        confidence = float(cand.confidence())

        # Vision bbox: normalized, bottom-left origin
        bb = obs.boundingBox()
        xn, yn, wn, hn = bb.origin.x, bb.origin.y, bb.size.width, bb.size.height

        # Convert to logical screen coordinates, top-left origin
        x = region_x + xn * logical_w
        y = region_y + (1.0 - yn - hn) * logical_h
        w = wn * logical_w
        h = hn * logical_h

        results.append({
            "text":       text,
            "x":          round(x),
            "y":          round(y),
            "w":          round(w),
            "h":          round(h),
            "cx":         round(x + w / 2),
            "cy":         round(y + h / 2),
            "confidence": round(confidence, 3),
        })

    print(json.dumps(results, ensure_ascii=False))

main()
"#;

pub fn main(args: &[String]) {
    let mut region: Option<(i32, i32, i32, i32)> = None;
    let mut image_path: Option<String> = None;
    let mut text_only = false;
    let mut min_confidence: f64 = 0.3;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--region" | "-r" if i + 1 < args.len() => {
                i += 1;
                let parts: Vec<i32> = args[i]
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                if parts.len() == 4 {
                    region = Some((parts[0], parts[1], parts[2], parts[3]));
                } else {
                    eprintln!("error: --region expects x,y,w,h");
                    return;
                }
            }
            "--text" | "-t" => text_only = true,
            "--min-confidence" | "-c" if i + 1 < args.len() => {
                i += 1;
                min_confidence = args[i].parse().unwrap_or(0.3);
            }
            "--help" | "-h" => {
                print_usage();
                return;
            }
            arg if !arg.starts_with('-') => {
                image_path = Some(arg.to_string());
            }
            _ => {}
        }
        i += 1;
    }

    // Capture screenshot if no image file given.
    let tmp_path = "/tmp/clx-ocr-input.png".to_string();
    let (img_path, region_x, region_y) = match image_path {
        Some(ref p) => (p.clone(), 0i32, 0i32),
        None => {
            let status = if let Some((x, y, w, h)) = region {
                std::process::Command::new("screencapture")
                    .args(["-x", "-t", "png", "-R", &format!("{},{},{},{}", x, y, w, h), &tmp_path])
                    .status()
            } else {
                std::process::Command::new("screencapture")
                    .args(["-x", "-t", "png", &tmp_path])
                    .status()
            };
            if !status.map(|s| s.success()).unwrap_or(false) {
                eprintln!("error: screencapture failed");
                return;
            }
            let (rx, ry) = region.map(|(x, y, _, _)| (x, y)).unwrap_or((0, 0));
            (tmp_path.clone(), rx, ry)
        }
    };

    // Write Python script to a temp file and run it.
    let py_path = "/tmp/clx-ocr.py";
    if std::fs::write(py_path, OCR_PYTHON).is_err() {
        eprintln!("error: could not write OCR script");
        return;
    }

    let output = std::process::Command::new("python3")
        .args([py_path, &img_path, &region_x.to_string(), &region_y.to_string()])
        .output();

    // Clean up temp screenshot.
    if image_path.is_none() {
        let _ = std::fs::remove_file(&tmp_path);
    }

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            eprintln!("error: python3 failed: {}", e);
            return;
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("error: OCR script failed:\n{}", stderr);
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout = stdout.trim();

    if text_only {
        // Parse JSON and print plain text lines.
        if let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(stdout) {
            for item in items {
                let conf = item["confidence"].as_f64().unwrap_or(0.0);
                if conf >= min_confidence {
                    if let Some(t) = item["text"].as_str() {
                        println!("{}", t);
                    }
                }
            }
        } else {
            eprintln!("error: could not parse OCR output: {}", stdout);
        }
        return;
    }

    // JSON output: filter by confidence and pretty-print.
    match serde_json::from_str::<Vec<serde_json::Value>>(stdout) {
        Ok(items) => {
            let filtered: Vec<_> = items
                .into_iter()
                .filter(|item| {
                    item["confidence"].as_f64().unwrap_or(0.0) >= min_confidence
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&filtered).unwrap_or_default());
        }
        Err(_) => {
            // Fallback: print raw output.
            println!("{}", stdout);
        }
    }
}

fn print_usage() {
    eprintln!("clx ocr — OCR a screenshot using Apple Vision Framework");
    eprintln!();
    eprintln!("Usage: clx ocr [options] [image_path]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --region x,y,w,h        Capture specific screen region");
    eprintln!("  --text                  Plain text output (no JSON)");
    eprintln!("  --min-confidence 0.5    Filter results below threshold (default: 0.3)");
    eprintln!("  --help                  Show this help");
    eprintln!();
    eprintln!("Output JSON fields (logical screen points, top-left origin):");
    eprintln!("  text, x, y, w, h, cx, cy, confidence");
    eprintln!("  cx/cy = center point, ready for: m <cx> <cy> c");
    eprintln!();
    eprintln!("Note: ~300ms cold start (Python subprocess). Slow layer use only.");
    eprintln!("      For fast layer (<30ms), use a persistent helper process.");
}
