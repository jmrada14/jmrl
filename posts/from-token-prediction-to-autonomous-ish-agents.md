---
title: From Token Prediction to Autonomous-ish Agents: Reflections on AI's Evolution and What Comes Next
date: 2026-01-02
description: From GPT-3 to coding agents, a holiday break with Claude Code, and what I think comes next.
---

My first interaction with a large language model was in November 2020. Back
then, working with OpenAI's GPT-3 meant describing a program in natural language,
receiving a mostly incorrect answer (unless you were asking for something trivial
like a hangman game), manually copying the code, compiling it, debugging, and
iterating endlessly. The human intervention required was immense.

Five years later, the landscape is unrecognizable.

## The arc of progress

What strikes me most isn't any single breakthrough. It's the compounding effect
of many. I've watched RAG systems rise and fade in relevance. I saw tool and
function calling emerge, laying the groundwork for protocols like MCP. But the
transformation that matters most to me is the evolution of coding agents: from
simple token predictors to systems that can perceive and interact with their
environment.

Today, using models like Anthropic's Claude Opus 4.5 or GPT-5.2-Codex through
interfaces designed for environment interaction (Claude Code, Codex CLI, Cursor,
or GitHub Copilot, which was the first I used and still use at work), the
experience is fundamentally different. Boilerplate gets handled. The agent
compiles and tests its own code. We're stepping away from "vibe coding" toward
systems that can produce quality software when guided properly.

Things aren't perfect. Software engineering as we know it will change, but it
won't disappear. Humans in the loop remain vital, especially in critical
industries where risk tolerance is low and the stakes are high.

## The enterprise reality, especially in finance

In most enterprise environments, and particularly in financial services, teams
are using tools like GitHub Copilot with access to frontier models like Claude
Opus 4.5. But they operate with significant restrictions, and rightfully so.

No one in a financially critical organization should want a non-deterministic
system prone to multiple attack surfaces, one that increases porosity
significantly, having access or control over operating systems, client data, or
infrastructure. The security risks are real. When it's someone else's money, you
don't get to move fast and break things.

Critical industries will always lag behind the frontier of innovation. That's not
a failure; it's prudent risk management. The key is mitigating risk as much as
possible while isolating how agents interact with systems and documents. This
allows enterprise to stay as close to the frontier as responsibly possible.

The financial sector has always moved cautiously with new technology, and for
good reason. That said, the distance between frontier and enterprise can shrink.
Smart isolation, proper guardrails, and thoughtful integration can bring these
tools into even the most regulated environments.

## Lessons from holiday break

During this holiday break, I finally had time to experiment with these tools
properly. I spent a lot of time with Claude Code in particular, and the results
exceeded my expectations.

With zero prior experience in Swift, I built full applications across Swift,
Rust, and JavaScript. I automated deployments and added quality gates throughout:
test cases, integration tests, all baked into pipelines where the agent could
iterate independently. I experimented with parallel agent usage through Vibe
Kanban.

The output? A few personal applications I'll actually use:

- **[Trapattoni](https://github.com/jmrada14/trapattoni)**: a native macOS/iOS
  football ⚽️ training companion with features for tracking my football practice
  and training sessions.
- **[Atelier](https://github.com/jmrada14/atelier)**: a fine art inventory system
  that tracks artwork, collectors, and even scrapes the web to find open calls for
  submissions.
- **[Limen](https://github.com/jmrada14/limen)**: a native Swift app for macOS to
  manage ports, processes, and network activity, including bulk-closing port
  connections (essential when Claude Code spins up my Vite server seven times).
- **[cc_validator](https://github.com/jmrada14/cc_validator)**: a Rust-based credit
  card validator with minimal dependencies, implementing the Luhn algorithm from
  scratch, exposed as a crate with Node.js bindings and compiled to WASM for
  browser use.

None of this would have been possible at this speed five years ago. Much of it
wouldn't have been possible even last year.

The total cost for all of these projects, plus additional research using Claude
and Claude Code? Around $342 USD. You could cut that significantly with a Max
subscription at $100 or $200/month. That's the price of a few hours of contractor
work for weeks of productive experimentation and multiple functional
applications.

## What I see coming

A few convictions have stuck with me.

**Terminal-based agents outperform IDE-based agents.** The closer to the
operating system, the easier it is to interact with the environment. This
architectural reality will shape tooling for years to come.

**Parallel agents will become the norm.** Multiple agents working on the same
codebase across different tasks will prove more efficient than tackling each
task sequentially. The bottleneck will shift to how fast teams can write specs,
define features, and break down requirements in ways that make sense to
implement.

**Code review gets harder, not easier.** Reviewing AI-generated code requires
more vigilance, not less. While many guardrails can be automated (extensive test
suites, static code analysis, CI gates), the biggest safeguard against incidents
will still be human review and testing. More tests and stricter rules, if done
right, reduce the slop.

**Spec writing, context engineering, and domain knowledge will be how engineers
stay relevant.** Systems will keep getting smarter, but knowing how and when to
guide them will remain an important skill.

**Jobs will reshape, not disappear.** While fear of replacement dominates news
headlines, the reshaping is already happening and moving fast. The imperative is
to ride the wave and use these tools as efficiently as possible. AI should
inspire creativity and help engineers, product managers, and business leaders
build better products faster than ever before.

But humans will remain responsible for providing context, managing agents, and
reviewing output. The judgment layer doesn't automate away.

## Looking ahead

The future is exciting. I hope to contribute both to pushing the frontier forward
and to adapting it responsibly for enterprise, especially in financial services
where so much is at stake.

The gap between what's possible and what's deployed in critical industries will
always exist. The opportunity lies in narrowing it thoughtfully, without
compromising what matters most: security, reliability, and trust.

The tools are here. What we do with them is up to us.
