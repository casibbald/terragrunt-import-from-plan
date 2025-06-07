### Architecture Overview

This document outlines key flows and relationships within the `terragrunt-import-from-plan` tool.

---

### 🧭 Plan-to-Import Sequence

```mermaid
sequenceDiagram
    participant User
    participant GitHubAction
    participant Terragrunt
    participant Terraform
    participant Simulator
    participant RustTool

    User->>Simulator: Define GCP infra using modules
    User->>Simulator: Setup dev config in /envs/simulator/dev

    GitHubAction->>Terragrunt: Run `terragrunt plan -out=tf.plan`
    Terragrunt->>Simulator: Evaluate configuration
    Terragrunt-->>GitHubAction: Outputs binary `tf.plan`

    GitHubAction->>Terraform: Run `terraform show -json tf.plan`
    Terraform-->>GitHubAction: Outputs `out.json`

    GitHubAction->>RustTool: Run import tool with CLI args

    RustTool->>RustTool: Parse modules.json
    RustTool->>RustTool: Parse out.json
    RustTool->>RustTool: Parse provider schema JSON (if available)
    RustTool->>RustTool: Map resources to modules
    RustTool->>RustTool: Infer resource IDs with ranking
    RustTool->>Terragrunt: Run `terragrunt import` per resource

    Terragrunt-->>RustTool: Import result
    RustTool-->>GitHubAction: Exit with success/failure
```

---

### 🧱 Rust Module Structure

```mermaid
graph TD
    main[main.rs] --> importer
    main --> plan
    importer --> utils
    utils --> importer
    importer --> plan
    importer --> errors
```

---

### 🔎 ID Inference Flow

```mermaid
flowchart TD
    Start --> HasSchema
    HasSchema -- Yes --> ExtractFromSchema
    HasSchema -- No --> HeuristicScan
    ExtractFromSchema --> RankFields
    HeuristicScan --> RankFields
    RankFields --> SelectCandidate
    SelectCandidate --> End
```

---

### 📦 Import Execution Flow

```mermaid
flowchart TD
    LoadPlan --> MapModules
    MapModules --> IterateResources
    IterateResources --> HasModule
    HasModule -- No --> SkipMissingMap
    HasModule -- Yes --> InferID
    InferID -- None --> SkipNoID
    InferID -- Some --> DryRunCheck
    DryRunCheck -- Yes --> PrintCommand
    DryRunCheck -- No --> ExecImport
```

---

### 🛠️ Terraform Schema Source

```mermaid
flowchart LR
    GitHubAction -->|generate| ProviderSchemaCmd[`terraform providers schema -json`]
    ProviderSchemaCmd --> FileOutput[`.terragrunt-provider-schema.json`]
    RustTool -->|load| FileOutput
    FileOutput --> IDScoring
```

This complements the `planned_values` schema and enables more precise attribute scoring for resource import ID inference.
