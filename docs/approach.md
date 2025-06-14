### Architecture Overview

This document outlines key flows and relationships within the `terragrunt-import-from-plan` tool.

---

### 🧭 Plan-to-Import Sequence

```mermaid
sequenceDiagram
    participant User
    participant GitHubAction
    participant RustTool
    participant Terragrunt
    participant FileSystem
    participant Simulator

    User->>Simulator: Define infrastructure using modules
    User->>Simulator: Setup config in /envs/simulator/[provider]/dev

    Note over User,GitHubAction: Live Environment (CI/CD)
    GitHubAction->>RustTool: Single Action: terragrunt-import-from-plan@v1.0.0
    Note over GitHubAction,RustTool: with: working-directory, plan-file
    
    Note over RustTool: All operations internal - from plan processing to import
    
    RustTool->>FileSystem: Process existing plan file
    RustTool->>RustTool: Generate modules.json dynamically
    RustTool->>RustTool: Parse plan JSON (from real .tfplan)
    RustTool->>RustTool: Parse provider schema JSON (if available)
    RustTool->>RustTool: Map resources to modules
    RustTool->>RustTool: Infer resource IDs with ranking
    
    alt Valid Real Infrastructure
        RustTool->>Terragrunt: Run `terragrunt import` per resource
        Terragrunt-->>RustTool: Import results
        RustTool-->>GitHubAction: ✅ Import summary with real infrastructure
    else Invalid/Missing Plan Data
        RustTool->>RustTool: FAIL IMMEDIATELY
        Note over RustTool: No synthetic JSON generation
        RustTool-->>GitHubAction: ❌ Error - real infrastructure required
    end
    
    GitHubAction-->>User: CI/CD results and artifacts
    
    Note over GitHubAction,RustTool: Development: Multi-step workflow for testing
    Note over RustTool: Dev: generate-fixtures + import commands separately
```

---

### 🧱 Rust Module Structure

```mermaid
graph TD
    main[main.rs<br/>CLI Entry Point] --> importer[importer.rs<br/>Core Import Logic]
    main --> plan[plan.rs<br/>Plan Processing]
    main --> utils[utils.rs<br/>**Core Orchestration**]
    
    utils --> |Internal Terragrunt Ops| terragrunt[Terragrunt CLI]
    utils --> |Fixture Generation| filesystem[File System]
    utils --> |Dynamic Discovery| modules[Module Discovery]
    utils --> |Real tfplan Conversion| conversion[JSON Conversion]
    
    importer --> utils
    importer --> plan
    plan --> utils
    
    utils --> schema[schema/<br/>Provider Schema Mgmt]
    utils --> scoring[scoring/<br/>ID Inference]
    
    importer --> errors[Error Handling]
    
    style utils fill:#74b9ff,stroke:#0984e3,stroke-width:3px
    style main fill:#00b894,stroke:#00a085,stroke-width:2px
```

---

### 🔧 Fixture Generation Workflow

```mermaid
flowchart TD
    Start[cargo run -- generate-fixtures provider] --> Clean[Clean workspace]
    Clean --> Init[terragrunt init --all]
    Init --> InitSuccess{Init Success?}
    InitSuccess -- No --> FailInit[Continue with warning<br/>Expected in CI]
    InitSuccess -- Yes --> Schema[Generate provider schema]
    FailInit --> Schema
    Schema --> Plan[terragrunt plan --all -out=out.tfplan]
    Plan --> PlanSuccess{Plan Success?}
    PlanSuccess -- No --> FailPlan[FAIL IMMEDIATELY<br/>Real infrastructure required]
    PlanSuccess -- Yes --> FindPlans[Find .tfplan files recursively]
    FindPlans --> Convert[terragrunt show -json out.tfplan]
    Convert --> ConvertSuccess{Conversion Success?}
    ConvertSuccess -- No --> FailConvert[FAIL IMMEDIATELY<br/>Invalid plan data]
    ConvertSuccess -- Yes --> GenModules[Generate modules.json dynamically]
    GenModules --> WriteFixtures[Write real fixtures to tests/fixtures/]
    WriteFixtures --> Success[✅ Real fixtures ready]
    
    style FailPlan fill:#ff6b6b,stroke:#d63031,stroke-width:3px
    style FailConvert fill:#ff6b6b,stroke:#d63031,stroke-width:3px
    style Success fill:#00b894,stroke:#00a085,stroke-width:2px
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
    Start[Tool Execution] --> ValidateReal{Real fixtures exist?}
    ValidateReal -- No --> FailFast[FAIL IMMEDIATELY<br/>No synthetic data]
    ValidateReal -- Yes --> LoadPlan[Load out.json from real .tfplan]
    LoadPlan --> LoadModules[Load dynamically generated modules.json]
    LoadModules --> MapModules[Map Resources to Modules]
    MapModules --> IterateResources[Iterate Resources]
    IterateResources --> HasModule{Has Module Mapping?}
    HasModule -- No --> SkipMissingMap[Skip - No Module Map]
    HasModule -- Yes --> InferID[Infer Resource ID]
    InferID -- None --> SkipNoID[Skip - No ID Inferred]
    InferID -- Some --> DryRunCheck{Dry Run Mode?}
    DryRunCheck -- Yes --> PrintCommand[Print Import Command]
    DryRunCheck -- No --> ExecImport[Execute Terragrunt Import]
    ExecImport --> IterateResources
    PrintCommand --> IterateResources
    SkipMissingMap --> IterateResources
    SkipNoID --> IterateResources
    
    style FailFast fill:#ff6b6b,stroke:#d63031,stroke-width:3px
    style LoadPlan fill:#00b894,stroke:#00a085,stroke-width:2px
    style LoadModules fill:#00b894,stroke:#00a085,stroke-width:2px
```

---

### 🛠️ Terraform Schema Source

```mermaid
flowchart LR
    RustTool -->|internal generate| ProviderSchemaCmd[`write_provider_schema()`]
    ProviderSchemaCmd --> FileOutput[`.terragrunt-provider-schema.json`]
    RustTool -->|load| FileOutput
    FileOutput --> IDScoring[ID Scoring & Inference]
```

This complements the `planned_values` schema and enables more precise attribute scoring for resource import ID inference. Schema generation is now handled internally by the Rust tool during fixture generation.
