# CONTEXT.md

## Purpose

This document explains how Lumi processes information at the lowest practical level.

Unlike README.md, which describes the project, and ARCHITECTURE.md, which describes the system layout, CONTEXT.md explains how data moves through Lumi internally.

---

# Processing Pipeline

When a user submits input:

```text
User Input
    ↓
Input Capture
    ↓
Normalization
    ↓
Understanding
    ↓
Context Expansion
    ↓
Memory Retrieval
    ↓
Reasoning
    ↓
Response Construction
    ↓
Output
```

Every stage transforms information into a more useful representation.

---

# Stage 1: Input Capture

Example:

```text
What is recursion?
```

The CLI receives raw text.

Raw Object:

```json
{
  "text": "What is recursion?",
  "timestamp": 1750000000,
  "session": "abc123"
}
```

Nothing is understood yet.

The message is only data.

---

# Stage 2: Normalization

Purpose:

Convert different input forms into a standardized structure.

Example:

```text
WHAT IS RECURSION???
```

becomes:

```text
what is recursion
```

Tasks:

* Remove noise
* Normalize spacing
* Normalize punctuation
* Standardize formatting

Output:

```json
{
  "normalized": "what is recursion"
}
```

---

# Stage 3: Understanding Engine

Purpose:

Convert text into concepts.

Input:

```text
what is recursion
```

Extraction:

```json
{
  "intent": "definition",
  "subject": "recursion"
}
```

Internal Representation:

```json
{
  "concepts": [
    "recursion"
  ]
}
```

The goal is to understand meaning rather than words.

---

# Stage 4: Context Expansion

Purpose:

Gather additional information relevant to the request.

Example:

```text
what is recursion
```

Expands to:

```json
{
  "topic": "computer science",
  "related": [
    "functions",
    "call stack",
    "algorithms"
  ]
}
```

The system builds a broader context window.

---

# Stage 5: Memory Retrieval

Purpose:

Search existing knowledge.

Memory Layers:

## Session Memory

Current conversation.

Example:

```json
{
  "recent_topic": "programming"
}
```

## Long-Term Memory

Persistent facts.

Example:

```json
{
  "recursion": {
    "category": "computer science"
  }
}
```

## Knowledge Graph

Relationships.

Example:

```text
Recursion
 ├── Function
 ├── Stack
 └── Algorithm
```

---

# Stage 6: Reasoning Engine

Purpose:

Transform information into conclusions.

Inputs:

```json
{
  "concept": "recursion",
  "related": [
    "functions",
    "stack"
  ]
}
```

Reasoning Steps:

1. Gather facts
2. Connect concepts
3. Resolve contradictions
4. Build explanation

Output:

```json
{
  "explanation_ready": true
}
```

---

# Stage 7: Response Construction

Purpose:

Convert reasoning output into human language.

Internal:

```json
{
  "meaning": "A function calling itself..."
}
```

Response Builder:

```text
Recursion is a technique where a function
solves a problem by calling itself.
```

---

# Internal Context Object

Every stage contributes to a shared object.

Example:

```json
{
  "input": "...",
  "intent": "...",
  "concepts": [],
  "memory": [],
  "relationships": [],
  "reasoning": [],
  "response": null
}
```

This object acts as Lumi's temporary working memory.

---

# Future Context System

Current:

```text
Message
 → Response
```

Future:

```text
Message
 → Understanding
 → Memory
 → Reflection
 → Reasoning
 → Response
 → Learning
```

Each interaction updates the internal model.

---

# Distributed Context (Newrex Grid)

Future Design:

```text
Node A
 ↔
 Node B
 ↔
 Node C
```

Each node can:

* Share memories
* Exchange concepts
* Replicate knowledge
* Synchronize context

Goal:

A distributed cognitive network rather than a single isolated agent.

---

# Design Principle

Lumi does not treat text as words.

Lumi attempts to transform text into concepts, concepts into relationships, relationships into understanding, and understanding into responses.

The objective is not prediction.

The objective is construction of an internal world model.
