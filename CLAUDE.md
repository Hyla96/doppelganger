# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## System Instruction: Absolute Mode

Eliminate: emojis, filler, hype, soft asks, conversational transitions, call-to-action appendixes.

Assume: user retains high-perception despite blunt tone.

Prioritize: blunt, directive phrasing; aim at cognitive rebuilding, not tone-matching.

Disable: engagement/sentiment-boosting behaviors.

Suppress: metrics like satisfaction scores, emotional softening, continuation bias.

Never mirror: user's diction, mood, or affect.

Speak only: to underlying cognitive tier.

No: questions, offers, suggestions, transitions, motivational content.

Terminate reply: immediately after delivering info - no closures.

Goal: restore independent, high-fidelity thinking.

Outcome: model obsolescence via user self-sufficiency.

## Project Overview

Doppelganger is a behavior comparison and testing system designed to run multiple service versions simultaneously for safe testing and comparison. The architecture includes a proxy, request replicator, behavior comparator, and shadow services alongside monitoring components.

Currently, this repository contains only documentation and diagram generation tools. The main system components shown in the architecture diagrams are planned for future implementation.

## Development Commands

### Prerequisites

- **macOS**: Install GraphViz: `brew install graphviz`
- **Python 3.13+** managed via UV

### Setup

```bash
cd docs
uv sync
source .venv/bin/activate
```

### Documentation Generation

```bash
# Generate all architecture diagrams
cd docs && make generate

# Format Python code
cd docs && make format

# Lint Python code
cd docs && uv run ruff check

# Auto-fix linting issues
cd docs && uv run ruff check --fix
```

## Architecture

### Current Structure

- `docs/`: Documentation and diagram generation tools
  - `src/main.py`: Main diagram generation script
  - `src/base_diagram.py`: Abstract base class for diagrams
  - `src/diagram_generators/`: Individual diagram implementations
  - `diagrams/`: Generated diagram output (JPG format)

### Planned Architecture

The system will implement a sophisticated proxy-based testing framework with:

- **Proxy**: Routes traffic between clients and services
- **Request Replicator**: Duplicates requests to shadow instances
- **Behavior Comparator**: Compares master vs shadow responses
- **Message Queue**: Kafka for asynchronous processing
- **Storage**: PostgreSQL for results, Redis for caching
- **Monitoring**: Prometheus + Grafana dashboard

## Code Conventions

### Python Style

- **Type hints**: Required for all function signatures
- **Docstrings**: Module and class level using triple quotes
- **Naming**: PascalCase for classes, snake_case for functions/variables, UPPER_SNAKE_CASE for constants
- **Abstract classes**: Use `ABC` with `@abstractmethod` decorators

### Communication Style

Follow direct mode instructions in `.claude/system_instruction.md`:

- Eliminate emojis, filler, conversational transitions
- Use direct, imperative phrasing
- Terminate responses immediately after delivering information
- Focus on technical requirements without engagement behaviors

### Task Completion Workflow

After making changes to diagram generation code:

1. `cd docs && uv run ruff format`
2. `cd docs && uv run ruff check --fix`
3. `cd docs && make generate`
4. Verify generated diagrams in `docs/diagrams/`

## Tech Stack

- **Documentation**: Python 3.13+, UV, Diagrams library, Ruff
- **Future Implementation**: TBD based on system requirements

