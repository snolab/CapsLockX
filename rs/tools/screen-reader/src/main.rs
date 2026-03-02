//! clx-screen-reader — reads text from all visible windows via UI Automation.
//!
//! Usage: clx-screen-reader.exe
//! Also available as: capslockx.exe read-screen-text

use windows::core::Interface;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Accessibility::*;

fn main() {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if let Err(e) = dump_screen_text() {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
        // Note: we intentionally skip CoUninitialize() here.
        // COM smart pointers (IUIAutomation, IUIAutomationElement, etc.) call
        // Release() in their Drop impls.  If CoUninitialize() runs first, those
        // Release() calls access-violate.  Since this is a short-lived CLI tool,
        // the OS reclaims everything on process exit anyway.
    }
}

unsafe fn dump_screen_text() -> Result<(), String> {
    let uia: IUIAutomation =
        CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| format!("CoCreateInstance: {e}"))?;

    let root = uia.GetRootElement().map_err(|e| format!("GetRootElement: {e}"))?;
    let cond = uia
        .CreateTrueCondition()
        .map_err(|e| format!("CreateTrueCondition: {e}"))?;

    let children = root
        .FindAll(TreeScope_Children, &cond)
        .map_err(|e| format!("FindAll: {e}"))?;
    let n = children.Length().unwrap_or(0);

    for i in 0..n {
        let elem = match children.GetElement(i) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let title = match elem.CurrentName() {
            Ok(b) => b.to_string(),
            Err(_) => continue,
        };
        if title.is_empty() {
            continue;
        }

        let offscreen = elem
            .CurrentIsOffscreen()
            .map(|b| b.as_bool())
            .unwrap_or(true);
        if offscreen {
            continue;
        }

        println!("\n=== [{}] ===", title);

        // Strategy 1: TextPattern — full text from editors / documents
        if let Ok(pat) = elem.GetCurrentPattern(UIA_TextPatternId) {
            if let Ok(tp) = pat.cast::<IUIAutomationTextPattern>() {
                if let Ok(range) = tp.DocumentRange() {
                    if let Ok(bstr) = range.GetText(-1) {
                        let s = bstr.to_string();
                        if !s.is_empty() {
                            println!("{s}");
                            continue;
                        }
                    }
                }
            }
        }

        // Strategy 2: walk descendants — collect Name + Value
        if let Ok(descs) = elem.FindAll(TreeScope_Descendants, &cond) {
            let dc = descs.Length().unwrap_or(0).min(300);
            for j in 0..dc {
                if let Ok(d) = descs.GetElement(j) {
                    if let Ok(nm) = d.CurrentName() {
                        let s = nm.to_string();
                        if !s.is_empty() {
                            println!("{s}");
                        }
                    }
                    if let Ok(pat) = d.GetCurrentPattern(UIA_ValuePatternId) {
                        if let Ok(vp) = pat.cast::<IUIAutomationValuePattern>() {
                            if let Ok(val) = vp.CurrentValue() {
                                let s = val.to_string();
                                if !s.is_empty() {
                                    println!("  [value: {s}]");
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
