# CONTEXT.md

# Lumi Cognitive Context Specification

Version: 0.1

Status: Experimental

---

# Overview

Context is the central mechanism that powers Lumi.

Everything inside Lumi exists as context.

Input is context.

Memory is context.

Knowledge is context.

Reasoning is context.

Responses are generated from context.

Unlike traditional systems that process text and immediately generate output, Lumi attempts to construct an internal representation of the world before generating a response.

---

# Core Principle

Traditional Systems

Input
→ Processing
→ Output

Lumi

Input
→ Understanding
→ Context Construction
→ Memory Integration
→ Reasoning
→ Response

The response is a byproduct.

Context is the primary objective.

---

# What Is Context?

Context is the temporary world model Lumi builds during processing.

Example:

User:

"What is recursion?"

Raw Text:

"What is recursion?"

Context Representation:

{
topic: "recursion",
category: "computer science",
intent: "definition",
confidence: 0.98
}

The text itself is not important.

The meaning extracted from the text is important.

---

# Internal Context Object

Every request creates a Context Object.

Example:

{
session: {},
input: {},
concepts: [],
entities: [],
relationships: [],
memories: [],
reasoning: [],
response: null
}

This object exists only during processing.

After completion important portions may be stored in memory.

---

# Processing Lifecycle

A request moves through multiple phases.

Phase 1:
Input Acquisition

Phase 2:
Normalization

Phase 3:
Understanding

Phase 4:
Context Construction

Phase 5:
Memory Retrieval

Phase 6:
Reasoning

Phase 7:
Response Synthesis

Phase 8:
Learning

---

# Phase 1: Input Acquisition

Purpose:

Receive information.

Sources:

* CLI
* API
* File
* Future GUI
* Future Voice Interface

Example:

lumi ask "What is recursion?"

Output:

{
source: "cli",
text: "What is recursion?"
}

---

# Phase 2: Normalization

Purpose:

Reduce noise.

Tasks:

* Remove duplicate spaces
* Normalize punctuation
* Normalize casing
* Expand contractions
* Repair malformed text

Example:

WHAT IS RECURSION???

becomes

what is recursion

---

# Phase 3: Understanding

Purpose:

Convert text into meaning.

Input:

what is recursion

Produces:

Intent:
Definition Request

Concept:
Recursion

Domain:
Computer Science

Confidence:
98%

Output:

{
intent: "define",
concept: "recursion"
}

---

# Concept Extraction

Concepts are the smallest meaningful units.

Example:

"The red car drives quickly"

Concepts:

Car
Drive

Attributes:

Red
Quickly

Internal Representation:

{
concepts: [
"car",
"drive"
]
}

---

# Entity Recognition

Entities represent identifiable objects.

Examples:

Satwik
Linux
OpenAI
India

Entities are stored separately from concepts.

Concept:
Country

Entity:
India

---

# Relationship Extraction

Meaning emerges from relationships.

Example:

"The cat sits on the chair"

Relationships:

Cat → sits_on → Chair

Stored as:

{
source: "cat",
relation: "sits_on",
target: "chair"
}

---

# Context Construction

After understanding, Lumi constructs a temporary world model.

Example:

Question:

"Why do birds fly?"

Context:

Bird
Wing
Flight
Aerodynamics
Evolution

Related concepts are activated.

The context becomes a miniature knowledge space.

---

# Context Expansion

Purpose:

Find additional relevant information.

Example:

Recursion

Automatically activates:

Function
Stack
Algorithm
Base Case
Tree Traversal

This allows deeper reasoning.

---

# Memory System

Memory provides continuity.

Without memory:

Each interaction is isolated.

With memory:

Knowledge accumulates.

---

# Memory Types

## Session Memory

Short-term memory.

Stores:

* Recent messages
* Current topic
* Temporary state

Lifetime:

Current session only.

---

## Working Memory

Active processing memory.

Contains:

* Current concepts
* Current reasoning chain
* Active entities

Destroyed after response generation.

---

## Semantic Memory

Stores knowledge.

Examples:

Earth is a planet.

Recursion is a programming technique.

Linux is an operating system.

---

## Episodic Memory

Stores events.

Examples:

User asked about recursion.

User created a compiler project.

User discussed operating systems.

---

## Procedural Memory

Stores processes.

Examples:

How to compile NXL.

How to start a project.

How to execute a task.

---

# Memory Retrieval

Purpose:

Find relevant memories.

Search Pipeline:

Input
↓
Concept Extraction
↓
Memory Search
↓
Ranking
↓
Selection

Only the most relevant memories enter context.

---

# Reasoning Engine

Purpose:

Transform knowledge into conclusions.

Reasoning consists of:

* Deduction
* Induction
* Analogy
* Synthesis

---

# Deductive Reasoning

Facts:

Birds can fly.

Sparrow is a bird.

Conclusion:

Sparrow can fly.

---

# Inductive Reasoning

Examples:

Dog has legs.

Cat has legs.

Horse has legs.

Conclusion:

Many animals have legs.

---

# Analogical Reasoning

Known:

CPU is like a brain.

New Concept:

Scheduler

Analogy:

Scheduler acts like traffic control.

---

# Synthesis

Combine multiple ideas.

Inputs:

Memory
Knowledge
Context

Output:

New explanation.

---

# Response Synthesis

Purpose:

Convert internal structures into language.

Input:

Structured knowledge.

Output:

Human-readable response.

Example:

Internal:

{
concept: "recursion",
relation: "self-reference"
}

Response:

Recursion is a technique where a function solves a problem by calling itself.

---

# Learning Layer

Future Component.

Purpose:

Improve internal models.

Steps:

1. Observe result
2. Evaluate quality
3. Update knowledge
4. Store useful information

---

# Reflection Layer

Future Component.

Purpose:

Self-review.

Questions:

Did the response answer the question?

Was reasoning correct?

Was memory useful?

Can understanding improve?

---

# Knowledge Graph

Future Component.

Knowledge is represented as nodes.

Example:

Programming
├── Algorithms
├── Data Structures
└── Recursion

Nodes connect through relationships.

The graph becomes Lumi's internal map of understanding.

---

# Context Compression

Large contexts cannot grow forever.

Compression converts:

Many details
↓
Summary
↓
Core meaning

This preserves knowledge while reducing size.

---

# Context Persistence

Important context may be stored.

Criteria:

* Frequently referenced
* High confidence
* Long-term relevance

---

# Failure States

Possible failures:

* Incorrect understanding
* Missing memory
* Weak reasoning
* Conflicting information

Recovery:

* Re-evaluate context
* Search additional memories
* Reconstruct reasoning chain

---

# Long-Term Vision

Lumi is not intended to be a chatbot.

Lumi is intended to become a cognitive system capable of:

* Understanding information
* Building internal models
* Maintaining memory
* Forming relationships
* Reasoning over knowledge
* Continuously improving understanding

Context is the foundation of every capability.

Everything Lumi becomes begins with context.
