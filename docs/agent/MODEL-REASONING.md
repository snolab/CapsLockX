# MODEL-REASONING — High-Quality Planning Models

Models optimized for complex reasoning, multi-step planning, and deep
scene understanding. Used for **strategy** — analyzing screenshots,
planning action sequences, and making high-level decisions.

These models are too slow for real-time command streaming but produce
higher-quality plans that the fast model then executes.

## Cloud Reasoning Models

### Frontier Models (best quality)

| Provider | Model | TTFT | tok/s | Vision | Context | Strength |
|---|---|---|---|---|---|---|
| Google | **Gemini 2.5 Pro** | 400-800ms | 80-130 | Yes | 1M | Best all-round, thinking mode |
| Google | **Gemini 2.5 Flash** (thinking) | 200-500ms | 120-180 | Yes | 1M | Fast reasoning with thinking budget |
| Anthropic | **Claude Opus 4** | 800-1500ms | 40-70 | Yes | 200K | Deepest reasoning |
| Anthropic | **Claude Sonnet 4** | 500-800ms | 60-90 | Yes | 200K | Best UI/computer use understanding |
| OpenAI | **GPT-4.1** | 300-600ms | 80-120 | Yes | 1M | Strong instruction following |
| OpenAI | **o3** | 1-10s | 30-60 | Yes | 200K | Extended thinking, highest quality |
| OpenAI | **o4-mini** | 500ms-3s | 50-80 | Yes | 200K | Budget reasoning model |

### IO Latency by Input Type

| Input type | Gemini 2.5 Pro | Claude Opus 4 | GPT-4.1 |
|---|---|---|---|
| Text only (500 tokens) | 400-800ms TTFT | 800-1500ms | 300-600ms |
| + 1 screenshot (768x768) | 600-1200ms TTFT | 1000-2000ms | 500-1000ms |
| + 3 screenshots | 800-1800ms TTFT | 1500-3000ms | 700-1500ms |
| Audio (10s via native) | 500-1000ms TTFT | N/A (no audio) | N/A (standard API) |

**Image tokens cost**: ~258-1500 tokens per image depending on resolution and provider tiling strategy. This adds to both latency and cost.

### Thinking / Extended Reasoning

| Model | Thinking mode | Thinking budget | Effect on latency |
|---|---|---|---|
| Gemini 2.5 Flash | Yes | Configurable (0-24576 tokens) | 0 = fast, high = 2-10s delay |
| Gemini 2.5 Pro | Yes | Configurable | Same |
| o3 | Yes (always) | Not configurable | 2-30s typical |
| o4-mini | Yes (always) | Not configurable | 1-10s typical |
| Claude Opus 4 | Extended thinking | Configurable budget | 1-30s |

For agent use: set thinking budget **low** (256-1024 tokens) for quick
plans, **high** (4096+) only for complex multi-step strategy.

## Local Reasoning Models

| Model | Params | tok/s (M2) | tok/s (4090) | Vision | Quality |
|---|---|---|---|---|---|
| Qwen 2.5 72B Q4 | 72B | 3-8 | 20-40 | No | Excellent |
| Llama 3.1 70B Q4 | 70B | 3-8 | 20-40 | No | Excellent |
| DeepSeek-V2-Lite | 16B | 10-20 | 30-60 | No | Good |
| LLaVA-NeXT 34B | 34B | 2-5 | 10-25 | **Yes** | Good vision |
| InternVL2 26B | 26B | 3-8 | 15-30 | **Yes** | Good vision |

Local 70B models are usable for planning at ~5 tok/s on M2 Max/Ultra.
Not fast enough for streaming commands, but fine for "analyze this
screenshot and give me a plan" (response in 2-5 seconds).

## Cloud Vision (Screenshot Analysis)

Specialized for understanding screenshots and UI:

| Model | Screenshot latency | UI understanding | Spatial precision | Notes |
|---|---|---|---|---|
| **Claude Sonnet 4** | 800ms-1.5s | **Best** | Good | Anthropic "computer use" heritage |
| **Gemini 2.5 Pro** | 600ms-1.2s | Excellent | Excellent | Thinking mode for complex UIs |
| GPT-4.1 | 500ms-1s | Good | Good | Reliable, well-documented |
| GPT-4o | 500ms-1.5s | Good | Good | |
| Claude Haiku 3.5 | 300-800ms | Good | Fair | Fast but less precise |

### Best practices for screenshot analysis
- Send at **768x768** max (diminishing returns beyond this)
- Use **JPEG quality 70-80%** (models handle compression well)
- **Annotate screenshots** with numbered labels on interactive elements
- **Crop to relevant region** instead of full screen when possible
- Include **structured context** alongside image: "I'm on the main menu, I want to start a new game"

## When to Use Reasoning Models

| Situation | Use reasoning? | Why |
|---|---|---|
| Simple key/mouse automation | No | Fast model handles directly |
| Complex multi-step workflow | **Yes** | Plan steps, then fast model executes |
| Unknown/new UI | **Yes** | Analyze screenshot, identify elements |
| Game strategy (what to do next) | **Yes** | Evaluate options, pick best action |
| Reaction to screen event | No | Too slow, use fast model + YOLO |
| Error recovery | **Yes** | Analyze what went wrong, replan |
| Learning new game mechanics | **Yes** | Understand rules from observation |

## Typical Usage Pattern

```
# 1. Reasoning model analyzes the situation (runs every 5-10s or on-demand)
[Send screenshot + context to reasoning model]
Response: "I see a login form. Steps:
  1. Click username field (coordinates ~300, 200)
  2. Type the username
  3. Click password field (~300, 260)
  4. Type the password
  5. Click 'Sign In' button (~300, 330)"

# 2. Fast model receives the plan + executes as CLX commands
System: "Execute this plan as CLX commands: [plan from above]"
Response (streaming):
m 300 200 c
w 200ms
k "myusername"
k tab
k "mypassword"
k ret
```

## Cost Comparison

| Provider | Model | Input $/M | Output $/M | $/screenshot |
|---|---|---|---|---|
| Google | Gemini 2.5 Pro | $1.25 | $10.00 | ~$0.002 |
| Google | Gemini 2.5 Flash | $0.15 | $0.60 | ~$0.0003 |
| Anthropic | Claude Opus 4 | $15.00 | $75.00 | ~$0.02 |
| Anthropic | Claude Sonnet 4 | $3.00 | $15.00 | ~$0.005 |
| OpenAI | GPT-4.1 | $2.00 | $8.00 | ~$0.003 |
| OpenAI | o3 | $10.00 | $40.00 | ~$0.015 |

At 1 screenshot every 5 seconds for 1 hour:
- Gemini 2.5 Flash: **~$0.22/hour** (best value reasoning)
- Claude Sonnet: **~$3.60/hour**
- o3: **~$10.80/hour**

## References

- [2025-03] [Gemini 2.5 Pro/Flash](https://ai.google.dev/gemini-api/docs/models) — Google AI model docs
- [2025-04] [GPT-4.1 announcement](https://openai.com/index/gpt-4-1/) — OpenAI blog
- [2025-02] [Claude model comparison](https://docs.anthropic.com/en/docs/about-claude/models) — Anthropic docs
- [2025-03] [o3/o4-mini reasoning models](https://openai.com/index/o3-and-o4-mini/) — OpenAI blog
- [2024-11] [Anthropic computer use](https://docs.anthropic.com/en/docs/agents-and-tools/computer-use) — Claude computer use docs
- [2024-06] [CRADLE: General Computer Agent](https://arxiv.org/abs/2403.03186) — game-playing agent with GPT-4V
- [2023-05] [Voyager: Minecraft LLM Agent](https://arxiv.org/abs/2305.16291) — open-ended Minecraft play
- [2024-03] [DeepMind SIMA](https://deepmind.google/discover/blog/sima-generalist-ai-agent-for-3d-virtual-environments/) — multi-game agent
