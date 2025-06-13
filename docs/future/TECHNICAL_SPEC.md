# 🌍 Terraform Drift Detection Bot — Product Technical Description

## 1. Overview

The **Terraform Drift Detection Bot** is an infrastructure automation tool that proactively monitors and alerts on configuration drift between cloud infrastructure and Terraform source code. It integrates with GitHub, supports multiple cloud providers (AWS, GCP, Azure), and provides both scheduled and real-time detection capabilities. It also features integrated ChatOps for on-call remediation directly from Slack or Microsoft Teams.

---

## 2. Goals

* Detect configuration drift as early as possible
* Eliminate undetected manual changes and config sprawl
* Maintain GitOps discipline with Terraform
* Empower on-call engineers to reconcile drift instantly via chat commands
* Support pluggable cloud backends (AWS, GCP, Azure)
* Reduce time-to-remediation (TTR) and infrastructure downtime

---

## 3. Bot Lifecycle & Triggers

### a. Trigger Modes

| Trigger Type      | Description                                                                                                                    |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| GitHub Schedule   | Executes `terraform plan` or `driftctl` at regular intervals (e.g., hourly) via GitHub Actions.                                |
| Cloud Push Events | Uses native cloud mechanisms (AWS Config, GCP Audit Logs, Azure Event Grid) to detect changes in real time and notify the bot. |
| Manual Command    | `/driftbot apply <env>` or button click in Teams/Slack triggers remediation via secure workflow dispatch.                      |

### b. Drift Detection Logic

1. Load latest Terraform code from the target repository.
2. Run `terraform init` and `terraform plan -detailed-exitcode` (or `driftctl scan`).
3. If exit code `2` or drift differences detected:

   * Parse the plan diff
   * Alert via Teams/Slack webhook
   * Include actionable summary, affected resources, and CTA for remediation
4. If drift is confirmed via chat command:

   * Trigger `terraform apply` workflow with audit logging

---

## 4. Architecture

### Core Components

| Component               | Description                                                                                   |
| ----------------------- | --------------------------------------------------------------------------------------------- |
| DriftBot Engine         | Main orchestrator for detection, reporting, and remediation.                                  |
| Drift Scanner           | Executes `terraform plan` or `driftctl scan`. Can run in GitHub Actions or on external infra. |
| ChatOps Gateway         | Listens for commands via Slack/Teams integration. Validates sender and dispatches actions.    |
| Notification Handler    | Sends alerts via webhook to target channels, includes formatted diff and action URL.          |
| GitHub Actions Executor | Executes scheduled or ad-hoc apply jobs with access to secrets and permissions.               |

### External Systems

* GitHub API (repos, workflows, secrets)
* Slack or Teams Webhook URL / Outgoing Bot
* Terraform-compatible cloud accounts (AWS, GCP, Azure)

---

## 5. Technical Stack

* **Language:** Go or TypeScript (cross-compatible with Probot or Bolt SDK)
* **Cloud Infra Access:**

  * AWS: IAM Role + AWS Config + CloudTrail
  * GCP: Workload Identity Federation + Pub/Sub
  * Azure: Azure Monitor + Event Grid + App Registration
* **CI:** GitHub Actions with OIDC for secure Terraform runs
* **Drift Detection:**

  * Native: `terraform plan -detailed-exitcode`
  * External: `driftctl scan --from tfstate://...`
* **Chat Integrations:**

  * Slack: Webhook + interactive buttons
  * Microsoft Teams: Bot Framework + message card actions

---

## 6. High-Level Epics

### EPIC 1: Baseline Drift Detection

* [ ] GitHub Action to run terraform plan
* [ ] Parse plan output and detect drift
* [ ] Exit with status and prepare webhook message

### EPIC 2: Cloud Event Subscriptions

* [ ] Configure AWS Config > SNS > Lambda/Webhook
* [ ] GCP Audit Log > Pub/Sub > HTTP push
* [ ] Azure Event Grid > Function App > HTTP trigger

### EPIC 3: Notification Engine

* [ ] Format drift reports
* [ ] Post to Slack and Teams with clickable CTA
* [ ] Include resource type, name, and diff summary

### EPIC 4: ChatOps Apply Handler

* [ ] Listen for `@driftbot apply <env>` command
* [ ] Validate on-call user identity
* [ ] Dispatch apply workflow to GitHub
* [ ] Confirm back to chat with outcome

### EPIC 5: Security & RBAC

* [ ] OIDC-based cloud auth
* [ ] Scoped secrets for drift vs apply workflows
* [ ] Approval gating before destructive actions

### EPIC 6: Reporting & Observability

* [ ] Track drift frequency per repo
* [ ] Store last drift event timestamp and duration
* [ ] Export metrics to Prometheus or Grafana

---

## 7. Sample User Stories

### 🧑‍💻 As a developer:

* I want to receive a drift alert if anyone modifies cloud resources outside Terraform.

### 👷 As a DevOps engineer:

* I want to view all drifted resources in Slack, with links to GitHub plan logs.
* I want to approve and apply Terraform from within Teams when drift is confirmed.

### 🧑‍🏫 As a platform team:

* I want to enforce that all infrastructure changes are declared in code.
* I want to know which teams are most frequently causing drift.

---

## 8. Success Metrics

* ⏱️ Median time to drift detection
* ✅ Drift remediation time (approval to apply)
* 🔁 Weekly drift event rate per project
* 🧑‍🔧 Number of user-triggered applies from chat
* 🛑 Prevented incidents due to early detection

---

## 9. Future Enhancements

* Auto-rollback or quarantine mode on high-risk drift
* GitHub Dashboard UI for managing detected drift
* Support for additional backends: Kubernetes, Helm, Pulumi
* Threat detection (e.g., resource created without tags or RBAC changed)

---

## 10. License

MIT © YourCompany. Enterprise support and integrations available upon request.
