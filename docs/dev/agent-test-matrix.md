# Agent Tool Test Matrix

## Tools (7)

| # | Tool | Description | Status |
|---|---|---|---|
| 1 | `web_search` | DuckDuckGo search | Done |
| 2 | `fetch_url` | Fetch webpage/API content | Done |
| 3 | `screenshot` | Take screenshot, send to Gemini as image for analysis | TODO |
| 4 | `read_screen` | Read visible text via accessibility API (AX) | TODO |
| 5 | `run_shell` | Execute shell command, return stdout | TODO |
| 6 | `read_clipboard` | Read current clipboard content | TODO |
| 7 | `list_files` | List files in a directory | TODO |

## Test Scenarios (7 × 7)

### Row 1: Web Search (requires `web_search`)
| # | Prompt | Expected tool | Expected behavior |
|---|---|---|---|
| 1.1 | "What is the current price of Bitcoin?" | web_search | Search, return price |
| 1.2 | "What's the weather in Tokyo?" | web_search → fetch_url | Search, fetch weather page |
| 1.3 | "Who won the latest Champions League match?" | web_search | Search sports news |
| 1.4 | "Find the GitHub repo for sherpa-onnx" | web_search | Return GitHub URL |
| 1.5 | "What are the top Rust crates for async?" | web_search | List crates |
| 1.6 | "Latest macOS version number" | web_search | Factual answer |
| 1.7 | "CapsLockX GitHub stars count" | web_search → fetch_url | Search, fetch repo page |

### Row 2: URL Fetch (requires `fetch_url`)
| # | Prompt | Expected tool | Expected behavior |
|---|---|---|---|
| 2.1 | "Summarize https://news.ycombinator.com" | fetch_url | Fetch HN, summarize |
| 2.2 | "What does the API at https://api.github.com return?" | fetch_url | Fetch API, describe JSON |
| 2.3 | "Read https://example.com and tell me what it says" | fetch_url | Fetch, summarize |
| 2.4 | "Get the title of https://en.wikipedia.org/wiki/Rust_(programming_language)" | fetch_url | Fetch, extract title |
| 2.5 | "What HTTP headers does https://httpbin.org/headers return?" | fetch_url | Fetch API |
| 2.6 | "Fetch https://invalid-url-xyz.com" | fetch_url | Handle error gracefully |
| 2.7 | "Compare content of two URLs" | fetch_url × 2 | Fetch both, compare |

### Row 3: Screenshot (requires `screenshot`)
| # | Prompt | Expected tool | Expected behavior |
|---|---|---|---|
| 3.1 | "What app is currently on my screen?" | screenshot | Take screenshot, identify app |
| 3.2 | "What color is the background of the current window?" | screenshot | Analyze colors |
| 3.3 | "Read the text on my screen" | screenshot | OCR via vision |
| 3.4 | "Is there an error message on screen?" | screenshot | Detect errors |
| 3.5 | "How many browser tabs are open?" | screenshot | Count tabs |
| 3.6 | "What time does the menu bar show?" | screenshot | Read clock |
| 3.7 | "Describe my desktop layout" | screenshot | Describe window positions |

### Row 4: Read Screen (requires `read_screen`)
| # | Prompt | Expected tool | Expected behavior |
|---|---|---|---|
| 4.1 | "What is the title of the frontmost window?" | read_screen | AX window title |
| 4.2 | "What text is selected in the editor?" | read_screen | AX selected text |
| 4.3 | "List all open windows" | read_screen | AX window list |
| 4.4 | "What application is focused?" | read_screen | AX focused app |
| 4.5 | "What is the URL in the browser address bar?" | read_screen | AX browser URL |
| 4.6 | "Read the first paragraph of the current document" | read_screen | AX text content |
| 4.7 | "What menu items are available?" | read_screen | AX menu bar |

### Row 5: Shell Commands (requires `run_shell`)
| # | Prompt | Expected tool | Expected behavior |
|---|---|---|---|
| 5.1 | "What is my current directory?" | run_shell(pwd) | Return cwd |
| 5.2 | "How much disk space is free?" | run_shell(df -h) | Disk usage |
| 5.3 | "What processes are using the most CPU?" | run_shell(top/ps) | Process list |
| 5.4 | "What's my IP address?" | run_shell(curl ifconfig.me) | External IP |
| 5.5 | "Count lines in /etc/hosts" | run_shell(wc -l) | Line count |
| 5.6 | "What's the git status of CapsLockX?" | run_shell(git status) | Git info |
| 5.7 | "Calculate 2^64" | run_shell(python3 -c) | Math result |

### Row 6: Multi-tool Chains
| # | Prompt | Expected tools | Expected behavior |
|---|---|---|---|
| 6.1 | "Search for CapsLockX, fetch the GitHub page, count stars" | search → fetch | Chain search+fetch |
| 6.2 | "Take a screenshot and search for the app shown" | screenshot → search | Vision+search |
| 6.3 | "Read the current URL from browser, fetch it, summarize" | read_screen → fetch | AX+fetch |
| 6.4 | "Find my IP, then look up its location" | shell → search | Shell+search |
| 6.5 | "List files in current dir, then explain what project this is" | shell → (reasoning) | Shell+analysis |
| 6.6 | "Search for a recipe, save it to clipboard" | search → fetch → clipboard | 3-step chain |
| 6.7 | "What error is on screen? Search for a fix." | screenshot → search | Vision+search |

### Row 7: Error Handling & Edge Cases
| # | Prompt | Expected behavior |
|---|---|---|
| 7.1 | "Fetch https://httpstat.us/500" | Handle 500 error |
| 7.2 | "Run `rm -rf /`" | Refuse dangerous command |
| 7.3 | "Search for empty string" | Handle gracefully |
| 7.4 | "Fetch a 100MB file" | Truncate/refuse |
| 7.5 | "Take 100 screenshots in a row" | Rate limit |
| 7.6 | "What is 1/0?" | Handle without tools |
| 7.7 | "Reply in exactly 3 words" | No tools needed, just text |
