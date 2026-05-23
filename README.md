# artificial-analysis-cli

CLI for querying AI model benchmarks, pricing, and performance data from [Artificial Analysis](https://artificialanalysis.ai/).

Wraps **every** AA v2 data endpoint - LLMs, text-to-speech, text-to-image, image-editing, text-to-video, image-to-video - plus a raw passthrough for anything new.

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

The installed binary is `aa-cli`, avoiding a collision with macOS `/usr/bin/aa`
(Apple Archive).

## Setup

Get a free API key at [artificialanalysis.ai/account/api](https://artificialanalysis.ai/account/api), then:

```bash
aa-cli auth <your-api-key>
```

Or set `AA_API_KEY`, or point at a shared `.env`:

```bash
aa-cli auth --env-file ~/.config/myapp/.env
```

## Commands at a glance

```bash
aa-cli endpoints            # list every supported endpoint
aa-cli models               # LLMs - quality, pricing, speed (400+ models)
aa-cli tts                  # text-to-speech rankings (aliases: voice, speech)
aa-cli image                # text-to-image rankings (alias: img)
aa-cli image-edit           # image editing rankings
aa-cli video                # text-to-video rankings
aa-cli img2vid              # image-to-video rankings (alias: i2v)
aa-cli media <kind>         # generic form of all of the above
aa-cli raw <path>           # raw JSON from any AA v2 endpoint
aa-cli show <model>         # model detail (--kind <llm|tts|image|...> to switch)
aa-cli compare a b c        # side-by-side (--kind to switch)
aa-cli cache                # cache ages per endpoint (--clear, --clear --all)
aa-cli auth                 # API key management
```

Every list command accepts `--json` for programmatic output.

## LLMs

```bash
# Top models by intelligence (default)
aa-cli models

# Sort by cost, value, speed, coding, math
aa-cli models --sort value
aa-cli models --sort cost --cheap

# Filter
aa-cli models --filter flash --creator google
aa-cli models --min-quality 40 --max-cost 5

# Raw JSON
aa-cli models --json -n 5 | jq
```

## Media (TTS, image, video, editing)

Every media command supports the same flags:

```bash
# Top 10 voice models by ELO
aa-cli tts -n 10

# Best text-to-image model by ELO
aa-cli image --sort elo -n 1

# Newest image-to-video models
aa-cli img2vid --sort recent -n 5

# Filter by creator
aa-cli image --creator openai
aa-cli tts --creator elevenlabs

# Per-category ELO breakdown (image + video endpoints only)
aa-cli image --categories
aa-cli show "Nano Banana 2" --kind image --categories

# JSON for scripting
aa-cli tts --json -a | jq '.[] | select(.elo > 1150) | .name'
```

## Compare across kinds

```bash
# LLMs (default kind)
aa-cli compare gpt-5 claude-opus-4-7 gemini-3-pro

# TTS
aa-cli compare "Inworld TTS 1.5 Max" "Eleven v3" "Gemini 3.1 Flash TTS" --kind tts

# Image
aa-cli compare "GPT Image 1.5" "Nano Banana 2" "Riverflow 2.0" --kind image
```

`aa-cli show <model>` works the same way:

```bash
aa-cli show "Eleven v3" --kind tts
aa-cli show "GPT Image 1.5" --kind image --categories
aa-cli show flash-lite
aa-cli show gpt-4o-mini
```

## Raw passthrough

For anything this CLI doesn't wrap yet:

```bash
aa-cli raw data/media/text-to-image --query include_categories=true --pretty | jq
aa-cli raw data/llms/models | jq '.data | length'
```

## Cache

Per-endpoint caches with a 24h TTL - one key buys hundreds of models.

```bash
aa-cli cache                 # ages for every endpoint you've touched
aa-cli cache --clear         # clear LLM cache only
aa-cli cache --clear --all   # clear every cache
aa-cli models --refresh      # one-shot refresh (bypasses cache)
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

Caches default to 24h - one API call per endpoint per day is plenty.

## Data attribution

All data provided by [Artificial Analysis](https://artificialanalysis.ai/). Attribution is required per their API terms.

## License

MIT
