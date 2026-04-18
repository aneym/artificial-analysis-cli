# artificial-analysis-cli

CLI for querying AI model benchmarks, pricing, and performance data from [Artificial Analysis](https://artificialanalysis.ai/).

Wraps **every** AA v2 data endpoint — LLMs, text-to-speech, text-to-image, image-editing, text-to-video, image-to-video — plus a raw passthrough for anything new.

## Install

```bash
cargo install --git https://github.com/aneym/artificial-analysis-cli
```

Or build from source:

```bash
git clone https://github.com/aneym/artificial-analysis-cli
cd artificial-analysis-cli
cargo install --path .
```

## Setup

Get a free API key at [artificialanalysis.ai/account/api](https://artificialanalysis.ai/account/api), then:

```bash
aa auth <your-api-key>
```

Or set `AA_API_KEY`, or point at a shared `.env`:

```bash
aa auth --env-file ~/.config/myapp/.env
```

## Commands at a glance

```bash
aa endpoints            # list every supported endpoint
aa models               # LLMs — quality, pricing, speed (400+ models)
aa tts                  # text-to-speech rankings (aliases: voice, speech)
aa image                # text-to-image rankings (alias: img)
aa image-edit           # image editing rankings
aa video                # text-to-video rankings
aa img2vid              # image-to-video rankings (alias: i2v)
aa media <kind>         # generic form of all of the above
aa raw <path>           # raw JSON from any AA v2 endpoint
aa show <model>         # model detail (--kind <llm|tts|image|...> to switch)
aa compare a b c        # side-by-side (--kind to switch)
aa cache                # cache ages per endpoint (--clear, --clear --all)
aa auth                 # API key management
```

Every list command accepts `--json` for programmatic output.

## LLMs

```bash
# Top models by intelligence (default)
aa models

# Sort by cost, value, speed, coding, math
aa models --sort value
aa models --sort cost --cheap

# Filter
aa models --filter flash --creator google
aa models --min-quality 40 --max-cost 5

# Raw JSON
aa models --json -n 5 | jq
```

## Media (TTS, image, video, editing)

Every media command supports the same flags:

```bash
# Top 10 voice models by ELO
aa tts -n 10

# Best text-to-image model by ELO
aa image --sort elo -n 1

# Newest image-to-video models
aa img2vid --sort recent -n 5

# Filter by creator
aa image --creator openai
aa tts --creator elevenlabs

# Per-category ELO breakdown (image + video endpoints only)
aa image --categories
aa show "Nano Banana 2" --kind image --categories

# JSON for scripting
aa tts --json -a | jq '.[] | select(.elo > 1150) | .name'
```

## Compare across kinds

```bash
# LLMs (default kind)
aa compare gpt-5 claude-opus-4-7 gemini-3-pro

# TTS
aa compare "Inworld TTS 1.5 Max" "Eleven v3" "Gemini 3.1 Flash TTS" --kind tts

# Image
aa compare "GPT Image 1.5" "Nano Banana 2" "Riverflow 2.0" --kind image
```

`aa show <model>` works the same way:

```bash
aa show "Eleven v3" --kind tts
aa show "GPT Image 1.5" --kind image --categories
```

## Raw passthrough

For anything this CLI doesn't wrap yet:

```bash
aa raw data/media/text-to-image --query include_categories=true --pretty | jq
aa raw data/llms/models | jq '.data | length'
```

## Cache

Per-endpoint caches with a 24h TTL — one key buys hundreds of models.

```bash
aa cache                 # ages for every endpoint you've touched
aa cache --clear         # clear LLM cache only
aa cache --clear --all   # clear every cache
aa models --refresh      # one-shot refresh (bypasses cache)
```

## API tiers

This CLI wraps the [Artificial Analysis API](https://artificialanalysis.ai/api-reference). The data available depends on your API tier:

| Feature                                          | Free       | Commercial |
| ------------------------------------------------ | ---------- | ---------- |
| LLM intelligence, coding, math indices           | Yes        | Yes        |
| Pricing (input/output per M tokens)              | Yes        | Yes        |
| Output speed & TTFT                              | Yes        | Yes        |
| Media ELO leaderboards (TTS, image, video, etc.) | Yes        | Yes        |
| Rate limit                                       | 25 req/day | Custom     |
| Prompt length benchmarks (10k, 100k)             | No         | Yes        |
| Provider-level benchmarks (Azure, Bedrock, etc.) | No         | Yes        |
| CritPt benchmark evaluation                      | 10 req/day | Custom     |

Caches default to 24h — one API call per endpoint per day is plenty.

## Data attribution

All data provided by [Artificial Analysis](https://artificialanalysis.ai/). Attribution is required per their API terms.

## License

MIT
