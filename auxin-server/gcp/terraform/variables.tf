# Variables for Auxin Server GCP deployment

variable "project_id" {
  description = "GCP Project ID"
  type        = string
}

variable "region" {
  description = "GCP region for deployment"
  type        = string
  default     = "us-central1"
}

variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
  default     = "dev"

  validation {
    condition     = contains(["dev", "staging", "prod"], var.environment)
    error_message = "Environment must be dev, staging, or prod."
  }
}

# Cloud Run Configuration
variable "cpu" {
  description = "Number of CPUs for Cloud Run instance"
  type        = string
  default     = "2"
}

variable "memory" {
  description = "Memory allocation for Cloud Run instance"
  type        = string
  default     = "2Gi"
}

variable "min_instances" {
  description = "Minimum number of Cloud Run instances"
  type        = number
  default     = 0
}

variable "max_instances" {
  description = "Maximum number of Cloud Run instances"
  type        = number
  default     = 10
}

variable "rust_log_level" {
  description = "Rust logging level"
  type        = string
  default     = "info,auxin_server=debug"
}

# Security Configuration
variable "auth_token_secret" {
  description = "Secret token for authentication (should be loaded from secure source)"
  type        = string
  sensitive   = true
}

variable "allow_public_access" {
  description = "Allow public access to Cloud Run service"
  type        = bool
  default     = false
}

variable "authorized_invokers" {
  description = "List of members authorized to invoke the service (if not public)"
  type        = list(string)
  default     = []
}

# Storage Configuration
variable "enable_versioning" {
  description = "Enable versioning on Cloud Storage bucket"
  type        = bool
  default     = true
}

# Network Configuration
variable "enable_vpc_connector" {
  description = "Enable VPC connector for private network access"
  type        = bool
  default     = false
}

# CI/CD Configuration
variable "enable_auto_deploy" {
  description = "Enable automatic deployment via Cloud Build triggers"
  type        = bool
  default     = false
}

variable "github_owner" {
  description = "GitHub repository owner (for Cloud Build triggers)"
  type        = string
  default     = ""
}

variable "github_repo" {
  description = "GitHub repository name (for Cloud Build triggers)"
  type        = string
  default     = ""
}
