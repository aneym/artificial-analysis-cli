# artificial-analysis-cli

CLI for querying AI model benchmarks, pricing, and performance data from [Artificial Analysis](https://artificialanalysis.ai/).

Compare 400+ models across intelligence scores, coding benchmarks, pricing, and speed — all from your terminal.

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

Or set the `AA_API_KEY` environment variable.

## Usage

### List models

```bash
# Top models by intelligence score (default)
aa models

# Sort by cost (cheapest first)
aa models --sort cost

# Sort by quality-per-dollar
aa models --sort value

# Sort by speed, coding, or math
aa models --sort speed
aa models --sort coding
aa models --sort math

# Budget models only (output price < $1/M tokens)
aa models --cheap

# Filter by name
aa models --filter "flash"
aa models --filter "gemini"

# Filter by creator
aa models --creator google
aa models --creator openai

# Combine filters
aa models --cheap --sort value --min-quality 15
aa models --creator anthropic --sort cost

# Show all models (default shows top 30)
aa models --all
```

### Compare models side-by-side

```bash
aa compare gpt-4o-mini claude-4-5-haiku flash-lite
aa compare "gpt-5" "claude-sonnet-4" "gemini-3-flash"
```

### Show model details

```bash
aa show flash-lite
aa show gpt-4o-mini
```

### Cache management

Data is cached for 24 hours to stay within rate limits.

```bash
aa cache          # Show cache age
aa cache --clear  # Force fresh data on next request
aa models --refresh  # One-time refresh
```

## API Tiers

This CLI wraps the [Artificial Analysis API](https://artificialanalysis.ai/api-reference). The data available depends on your API tier:

| Feature                                          | Free       | Commercial |
| ------------------------------------------------ | ---------- | ---------- |
| LLM intelligence, coding, math indices           | Yes        | Yes        |
| Pricing (input/output per M tokens)              | Yes        | Yes        |
| Output speed & TTFT                              | Yes        | Yes        |
| Rate limit                                       | 25 req/day | Custom     |
| Prompt length benchmarks (10k, 100k)             | No         | Yes        |
| Provider-level benchmarks (Azure, Bedrock, etc.) | No         | Yes        |
| CritPt benchmark evaluation                      | 10 req/day | Custom     |

The free tier is sufficient for most use cases — the CLI caches aggressively (24h TTL) so a single API call fetches all 400+ models.

For commercial API access, contact [hello@artificialanalysis.ai](mailto:hello@artificialanalysis.ai).

## Data Attribution

All data provided by [Artificial Analysis](https://artificialanalysis.ai/). Attribution is required per their API terms.

## Contributing

PRs welcome! Some ideas:

- [ ] JSON output mode (`--json`) for piping
- [ ] CSV export
- [ ] Model recommendation engine (best model for budget + task type)
- [ ] Eval harness integration (run your own benchmarks, compare to AA scores)
- [ ] Provider-level pricing comparison
- [ ] Historical trend tracking

## License

MIT
