# GEMINI.MD: AI Collaboration Guide

This document provides essential context for AI models interacting with this project. Adhering to these guidelines will ensure consistency and maintain code quality.

## 1. Project Overview & Purpose

* **Primary Goal:** This project aims to implement a Peer-to-Peer VPN solution.
* **Business Domain:** Network Infrastructure, Security.

## 2. Core Technologies & Stack

* **Languages:** Rust, Typst (for documentation).
* **Frameworks & Runtimes:** Rust runtime.
* **Key Libraries/Dependencies:** `sops` (Secrets OPerationS).
* **Package Manager(s):** Cargo.

## 3. Architectural Patterns

* **Overall Architecture:** General application. Specific architectural patterns (e.g., client-server, microservices) are not yet discernible from the current file structure.
* **Directory Structure Philosophy:**
  * `/`: Project root, contains configuration files and documentation.
  * `.direnv/`: direnv environment management files.
  * `src/`: primary source code.

## 4. Coding Conventions & Style Guide

* **Formatting:** Adhere to standard Rust formatting. No explicit configuration files found.
* **Naming Conventions:** Follow idiomatic Rust naming conventions (snake_case for functions/variables, PascalCase for types/modules).
* **API Design:** Not applicable at this stage, or not yet defined.
* **Error Handling:** Follow idiomatic Rust error handling practices (e.g., `Result` enum, `anyhow`, `thiserror`).

## 5. Key Files & Entrypoints

* **Main Entrypoint(s):** `src/main.rs`.
* **Configuration:** `default.nix` (Nix shell configuration), `.env` (direnv environment variables).

## 6. Development & Testing Workflow

* **Local Development Environment:** The project uses `direnv` and `nix-shell` (via `default.nix`) for managing the development environment and dependencies. To set up, ensure `direnv` is hooked into your shell and `nix` is installed. The environment will load automatically upon entering the project directory.
* **Testing:** Standard Rust testing practices (e.g., `cargo test`). New code should include corresponding unit and integration tests.

## 7. Specific Instructions for AI Collaboration

* **Contribution Guidelines:** No explicit `CONTRIBUTING.md` found. Follow standard open-source contribution practices: create feature branches, submit pull requests, and ensure code is well-tested.
* **Security:** Always prioritize security. Do not hardcode sensitive information. Ensure any network-related changes are secure and properly vetted.
* **Dependencies:** When adding new dependencies, update `default.nix` for Nix packages and `Cargo.toml` for Rust dependencies.
* **Commit Messages:** Follow a clear and descriptive commit message style. Use Conventional Commits for consistency.
